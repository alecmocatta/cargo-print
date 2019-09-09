//! A cargo subcommand to print information in a shell-convenient format.
//!
//! **[Crates.io](https://crates.io/crates/cargo-print) │ [Repo](https://github.com/alecmocatta/cargo-print)**

#![doc(html_root_url = "https://docs.rs/cargo-print/0.1.1")]
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
#![allow(clippy::useless_let_if_seq, clippy::if_not_else)]

use cargo_metadata::{CargoOpt, MetadataCommand};
use std::{
	collections::{HashMap, HashSet, VecDeque}, env, process
};

fn main() {
	let mut args = env::args().skip(2);
	let mut opt_no_default_features = false;
	let mut opt_features = HashSet::new();
	let mut opt_all_features = false;
	let mut error = false;
	if args.next().as_ref().map(String::as_str) != Some("examples") {
		error = true;
	}
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
