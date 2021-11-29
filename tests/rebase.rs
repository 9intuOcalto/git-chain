use console;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;
use std::process::Output;

pub mod common;
use common::{
    checkout_branch, commit_all, create_branch, create_new_file, display_outputs, first_commit_all,
    generate_path_to_repo, get_current_branch_name, run_test_bin, run_test_bin_expect_err,
    run_test_bin_expect_ok, setup_git_repo, teardown_git_repo,
};

fn run_test_bin_for_rebase<I, T, P: AsRef<Path>>(current_dir: P, arguments: I) -> Output
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    let output = run_test_bin(current_dir, arguments);

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }

    assert!(output.status.success());

    // https://git-scm.com/docs/git-rebase#_miscellaneous_differences
    // git rebase will output to both stdout and stderr.

    output
}

#[test]
fn rebase_subcommand_simple() {
    let repo_name = "rebase_subcommand_simple";
    let repo = setup_git_repo(repo_name);
    let path_to_repo = generate_path_to_repo(repo_name);

    {
        // create new file
        create_new_file(&path_to_repo, "hello_world.txt", "Hello, world!");

        // add first commit to master
        first_commit_all(&repo, "first commit");
    };

    assert_eq!(&get_current_branch_name(&repo), "master");

    // create and checkout new branch named some_branch_1
    {
        let branch_name = "some_branch_1";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_1");

        // create new file
        create_new_file(&path_to_repo, "file_1.txt", "contents 1");

        // add commit to branch some_branch_1
        commit_all(&repo, "message");
    };

    // create and checkout new branch named some_branch_2
    {
        let branch_name = "some_branch_2";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_2");

        // create new file
        create_new_file(&path_to_repo, "file_2.txt", "contents 2");

        // add commit to branch some_branch_2
        commit_all(&repo, "message");
    };

    // create and checkout new branch named some_branch_3
    {
        let branch_name = "some_branch_3";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_3");

        // create new file
        create_new_file(&path_to_repo, "file_3.txt", "contents 3");

        // add commit to branch some_branch_3
        commit_all(&repo, "message");
    };

    // create and checkout new branch named some_branch_2.5
    {
        checkout_branch(&repo, "some_branch_2");
        let branch_name = "some_branch_2.5";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_2.5");

        // create new file
        create_new_file(&path_to_repo, "file_2.5.txt", "contents 2.5");

        // add commit to branch some_branch_2.5
        commit_all(&repo, "message");
    };

    // create and checkout new branch named some_branch_1.5
    {
        checkout_branch(&repo, "some_branch_1");
        let branch_name = "some_branch_1.5";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_1.5");

        // create new file
        create_new_file(&path_to_repo, "file_1.5.txt", "contents 1.5");

        // add commit to branch some_branch_1.5
        commit_all(&repo, "message");
    };

    // create and checkout new branch named some_branch_0
    {
        checkout_branch(&repo, "master");
        let branch_name = "some_branch_0";
        create_branch(&repo, branch_name);
        checkout_branch(&repo, branch_name);
    };

    {
        assert_eq!(&get_current_branch_name(&repo), "some_branch_0");

        // create new file
        create_new_file(&path_to_repo, "file_0.txt", "contents 0");

        // add commit to branch some_branch_0
        commit_all(&repo, "message");
    };

    assert_eq!(&get_current_branch_name(&repo), "some_branch_0");

    // run git chain setup
    let args: Vec<&str> = vec![
        "setup",
        "chain_name",
        "master",
        "some_branch_0",
        "some_branch_1",
        "some_branch_1.5",
        "some_branch_2",
        "some_branch_2.5",
        "some_branch_3",
    ];
    let output = run_test_bin_expect_ok(&path_to_repo, args);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"
🔗 Succesfully set up chain: chain_name

chain_name
      some_branch_3 ⦁ 1 ahead ⦁ 1 behind
      some_branch_2.5 ⦁ 1 ahead
      some_branch_2 ⦁ 1 ahead ⦁ 1 behind
      some_branch_1.5 ⦁ 1 ahead
      some_branch_1 ⦁ 1 ahead ⦁ 1 behind
    ➜ some_branch_0 ⦁ 1 ahead
      master (root branch)
"#
        .trim_start()
    );

    // git rebase
    let args: Vec<&str> = vec!["rebase"];
    let output = run_test_bin_for_rebase(&path_to_repo, args);

    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("Current branch some_branch_0 is up to date."));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Switching back to branch: some_branch_0")
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("🎉 Successfully rebased chain chain_name"));

    assert_eq!(
        console::strip_ansi_codes(&String::from_utf8_lossy(&output.stderr))
            .trim()
            .replace("\r", "\n"),
        "
Rebasing (1/1)

Successfully rebased and updated refs/heads/some_branch_1.
Rebasing (1/1)

Successfully rebased and updated refs/heads/some_branch_1.5.
Rebasing (1/1)

Successfully rebased and updated refs/heads/some_branch_2.
Rebasing (1/1)

Successfully rebased and updated refs/heads/some_branch_2.5.
Rebasing (1/1)

Successfully rebased and updated refs/heads/some_branch_3.
"
        .trim()
    );

    // git chain
    let args: Vec<&str> = vec![];
    let output = run_test_bin_expect_ok(&path_to_repo, args);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"
On branch: some_branch_0

chain_name
      some_branch_3 ⦁ 1 ahead
      some_branch_2.5 ⦁ 1 ahead
      some_branch_2 ⦁ 1 ahead
      some_branch_1.5 ⦁ 1 ahead
      some_branch_1 ⦁ 1 ahead
    ➜ some_branch_0 ⦁ 1 ahead
      master (root branch)
"#
        .trim_start()
    );

    // git rebase
    let args: Vec<&str> = vec!["rebase"];
    let output = run_test_bin_expect_ok(&path_to_repo, args);

    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Switching back to branch: some_branch_0")
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Chain chain_name is already up-to-date.")
    );

    teardown_git_repo(repo_name);
}
