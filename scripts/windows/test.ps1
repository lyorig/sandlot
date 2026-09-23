<#
    .SYNOPSIS
    Runs tests.

    .DESCRIPTION
    This is a wrapper script that runs `cargo test` with settings required for this crate.
    SDL requires most things to run on the main thread, which can be done, but it's a hassle
    to repeat the required flags every time you want to test.
#>

cargo test --tests -- --test-threads=1
