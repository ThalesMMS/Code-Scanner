use code_scanner::cli::Args;
use code_scanner::scanner::{collect_loc_stats, process_project};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn collect_loc_stats_counts_lines_and_skips_ignored() {
    let dir = tempdir().expect("temp dir");
    let root = dir.path();

    fs::write(root.join("main.rs"), "line1\nline2\nline3\n").expect("write main");
    fs::write(root.join("notes.txt"), "a\nb\n").expect("write notes");
    fs::write(root.join("yarn.lock"), "lock").expect("write lock");
    fs::write(root.join("image.png"), [137u8, 80, 78, 71]).expect("write png");
    fs::write(root.join("binary.rs"), [0u8, 1, 2]).expect("write binary");

    let ignored_dir = root.join("node_modules");
    fs::create_dir_all(&ignored_dir).expect("create node_modules");
    fs::write(ignored_dir.join("ignored.js"), "ignored\n").expect("write ignored");

    let args = Args {
        input_dir: PathBuf::from("."),
        output_dir: PathBuf::from("."),
        loc: None,
        no_gitignore: true,
        verbose: false,
        ignore: vec![],
        max_output_lines: None,
        lang: None,
    };

    let stats = collect_loc_stats(root, &args).expect("loc stats");

    assert_eq!(stats.processed_files, 2);
    assert_eq!(stats.skipped_files, 3);
    assert_eq!(stats.total_lines, 5);
    assert_eq!(stats.total_chars, 22);
    assert_eq!(stats.estimated_tokens, 6);
    assert_eq!(stats.largest_files[0].lines, 3);
    assert_eq!(stats.largest_files[1].lines, 2);
}

#[test]
fn split_output_rotates_before_next_source_file() {
    let dir = tempdir().expect("temp dir");
    let project = dir.path().join("sample");
    let output = dir.path().join("out");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&output).expect("create output");

    fs::write(project.join("alpha.rs"), "alpha-line-1\nalpha-line-2\n").expect("write alpha");
    fs::write(project.join("beta.rs"), "beta-line-1\nbeta-line-2\n").expect("write beta");
    fs::write(project.join("gamma.rs"), "gamma-line-1\ngamma-line-2\n").expect("write gamma");

    let args = Args {
        input_dir: project.clone(),
        output_dir: output.clone(),
        loc: None,
        no_gitignore: true,
        verbose: false,
        ignore: vec![],
        max_output_lines: Some(8),
        lang: None,
    };

    process_project(&project, &output, &args).expect("process project");

    let part1 = fs::read_to_string(output.join("sample_project_code.txt")).expect("read part 1");
    let part2 = fs::read_to_string(output.join("sample_project_code_2.txt")).expect("read part 2");

    assert!(
        part1.contains("alpha-line-1") && part1.contains("alpha-line-2"),
        "part 1 must contain alpha.rs in full"
    );
    assert!(
        part1.contains("beta-line-1") && part1.contains("beta-line-2"),
        "part 1 must contain beta.rs in full"
    );
    assert!(
        !part1.contains("gamma-line-1"),
        "part 1 must not contain gamma.rs once the line limit is exceeded"
    );
    assert!(
        part2.contains("gamma-line-1") && part2.contains("gamma-line-2"),
        "part 2 must contain gamma.rs in full"
    );
}

#[test]
fn split_output_keeps_oversized_source_file_in_one_part() {
    let dir = tempdir().expect("temp dir");
    let project = dir.path().join("sample");
    let output = dir.path().join("out");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&output).expect("create output");

    let large_content = (1..=20)
        .map(|n| format!("large-line-{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(project.join("large.rs"), format!("{large_content}\n")).expect("write large");
    fs::write(project.join("tail.rs"), "tail-line-1\ntail-line-2\n").expect("write tail");

    let args = Args {
        input_dir: project.clone(),
        output_dir: output.clone(),
        loc: None,
        no_gitignore: true,
        verbose: false,
        ignore: vec![],
        max_output_lines: Some(5),
        lang: None,
    };

    process_project(&project, &output, &args).expect("process project");

    let part1 = fs::read_to_string(output.join("sample_project_code.txt")).expect("read part 1");
    let part2 = fs::read_to_string(output.join("sample_project_code_2.txt")).expect("read part 2");

    for n in 1..=20 {
        let marker = format!("large-line-{n}");
        assert!(
            part1.contains(&marker) && !part2.contains(&marker),
            "large.rs line {n} must stay in part 1 only"
        );
    }
    assert!(
        part2.contains("tail-line-1") && part2.contains("tail-line-2"),
        "tail.rs must be written entirely to part 2"
    );
}
