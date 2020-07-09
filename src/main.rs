//! A cargo subcommand to print information in a shell-convenient format.
//!
//! **[Crates.io](https://crates.io/crates/cargo-print) │ [Repo](https://github.com/alecmocatta/cargo-print)**

#![doc(html_root_url = "https://docs.rs/cargo-print/0.1.3")]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
#![allow(clippy::if_not_else, clippy::too_many_lines)]

use cargo_metadata::{CargoOpt, MetadataCommand};
use std::{
	collections::{HashMap, HashSet, VecDeque}, env, process
};

fn main() {
	let mut args = env::args().skip(2);
	match args.next().as_deref() {
		Some("examples") => print_examples(args),
		Some("publish") => print_publish(args),
		Some("package") => print_package(args),
		Some("directory") => print_directory(args),
		_ => {
			eprintln!("USAGE:\n    cargo print examples [--no-default-features] [--features <FEATURES>...] [--all-features]\n    cargo print publish\n    cargo print package\n    cargo print directory <package-name>");
			process::exit(1);
		}
	}
}

fn print_directory(mut args: impl Iterator<Item = String>) {
	let package_name = if let (Some(package_name), None) = (args.next(), args.next()) {
		package_name
	} else {
		eprintln!("USAGE:\n    cargo print directory");
		process::exit(1);
	};
	let metadata = MetadataCommand::new().exec().unwrap();
	let package = metadata
		.packages
		.into_iter()
		.filter(|package| package.name == package_name)
		.collect::<Vec<_>>();
	assert!(package.len() <= 1);
	if package.is_empty() {
		panic!("package {} not found", package_name);
	}
	let package = package.into_iter().next().unwrap();
	let mut manifest_path = package.manifest_path;
	let _ = manifest_path.pop();
	println!("{}", manifest_path.display());
}

fn print_package(mut args: impl Iterator<Item = String>) {
	if args.next().is_some() {
		eprintln!("USAGE:\n    cargo print package");
		process::exit(1);
	}
	let current_dir = env::current_dir().unwrap();
	let current_manifest = current_dir.join("Cargo.toml");
	let metadata = MetadataCommand::new().exec().unwrap();
	let package = metadata
		.packages
		.into_iter()
		.filter(|package| package.manifest_path == current_manifest)
		.collect::<Vec<_>>();
	if package.len() > 1 {
		panic!("We seem to be in > 1 package {:?}", package);
	}
	if package.is_empty() {
		panic!("We don't seem to be in a package");
	}
	let package = package.into_iter().next().unwrap();
	println!("{}", package.name);
}

fn print_publish(mut args: impl Iterator<Item = String>) {
	if args.next().is_some() {
		eprintln!("USAGE:\n    cargo print publish");
		process::exit(1);
	}
	let metadata = MetadataCommand::new().exec().unwrap();
	let members = metadata
		.workspace_members
		.into_iter()
		.collect::<HashSet<_>>();
	let members = metadata
		.packages
		.into_iter()
		.filter_map(|package| {
			if members.contains(&package.id) {
				Some((package.name.clone(), package))
			} else {
				None
			}
		})
		.collect::<HashMap<_, _>>();
	let mut members: HashMap<String, HashSet<String>> = members
		.iter()
		.map(|(name, package)| {
			(
				name.clone(),
				package
					.dependencies
					.iter()
					.filter_map(|dep| {
						if members.contains_key(&dep.name) {
							Some(dep.name.clone())
						} else {
							None
						}
					})
					.collect::<HashSet<String>>(),
			)
		})
		.collect::<HashMap<_, _>>();
	let mut dependents: HashMap<String, HashSet<String>> = HashMap::new();
	for (package, dependencies) in &members {
		for dependency in dependencies {
			let _ = dependents
				.entry(dependency.to_owned())
				.or_insert_with(HashSet::new)
				.insert(package.to_owned());
		}
	}
	while !members.is_empty() {
		let publish = members
			.iter()
			.find(|(_member, dependencies)| dependencies.is_empty())
			.expect("circular dependencies")
			.0
			.clone();
		println!("{}", publish);
		let _ = members.remove(&publish).unwrap();
		for dependent in dependents.get(&publish).unwrap_or(&HashSet::new()) {
			let _ = members.get_mut(dependent).unwrap().remove(&publish);
		}
	}
}

fn print_examples(mut args: impl Iterator<Item = String>) {
	let mut opt_no_default_features = false;
	let mut opt_features = HashSet::new();
	let mut opt_all_features = false;
	let mut error = false;
	while let Some(arg) = args.next() {
		match &*arg {
			"--no-default-features" => opt_no_default_features = true,
			"--features" => {
				if let Some(features) = args.next() {
					for feature in features.split(' ').filter(|feature| !feature.is_empty()) {
						let _ = opt_features.insert(feature.to_owned());
					}
				} else {
					error = true
				}
			}
			"--all-features" => opt_all_features = true,
			_ => error = true,
		}
	}
	if error {
		eprintln!("USAGE:\n    cargo print examples [--no-default-features] [--features <FEATURES>...] [--all-features]");
		process::exit(1);
	}
	let current_dir = env::current_dir().unwrap();
	let current_manifest = current_dir.join("Cargo.toml");
	let mut metadata = MetadataCommand::new();
	// TODO: what do these args actually do?
	if opt_no_default_features {
		let _ = metadata.features(CargoOpt::NoDefaultFeatures);
	}
	let _ = metadata.features(CargoOpt::SomeFeatures(
		opt_features.iter().cloned().collect(),
	));
	if opt_all_features {
		let _ = metadata.features(CargoOpt::AllFeatures);
	}
	let metadata = metadata.exec().unwrap();
	let package = metadata
		.packages
		.into_iter()
		.filter(|package| package.manifest_path == current_manifest)
		.collect::<Vec<_>>();
	if package.len() > 1 {
		panic!("We seem to be in > 1 package {:?}", package);
	}
	if package.is_empty() {
		panic!("We don't seem to be in a package");
	}
	let package = package.into_iter().next().unwrap();
	let features = package
		.dependencies
		.into_iter()
		.filter_map(|dep| {
			if dep.optional {
				Some((dep.rename.unwrap_or(dep.name), Vec::new()))
			} else {
				None
			}
		})
		.chain(package.features.into_iter())
		.collect::<HashMap<String, Vec<String>>>();
	let features_set = features.keys().cloned().collect::<HashSet<_>>();
	let invalid_features = opt_features.difference(&features_set).collect::<Vec<_>>();
	if !invalid_features.is_empty() {
		println!("invalid feature {:?}", invalid_features);
		process::exit(1);
	}
	let mut enabled_features;
	if !opt_all_features {
		enabled_features = opt_features;
		if !opt_no_default_features && features.contains_key("default") {
			let _ = enabled_features.insert(String::from("default"));
		}
	} else {
		enabled_features = features.iter().map(|(k, _)| k.clone()).collect();
	}
	let mut features_stack = enabled_features
		.clone()
		.into_iter()
		.collect::<VecDeque<_>>();
	while let Some(feature) = features_stack.pop_front() {
		for feature in features.get(&feature).unwrap_or(&Vec::new()).iter() {
			if enabled_features.insert(feature.clone()) {
				features_stack.push_back(feature.clone())
			}
		}
	}
	let examples = package
		.targets
		.into_iter()
		.filter_map(|target| {
			if target.kind.contains(&String::from("example"))
				&& target
					.required_features
					.iter()
					.cloned()
					.collect::<HashSet<_>>()
					.difference(&enabled_features)
					.count() == 0
			{
				Some(target.name)
			} else {
				None
			}
		})
		.collect::<HashSet<_>>();
	for example in examples {
		println!("{}", example);
	}
}
