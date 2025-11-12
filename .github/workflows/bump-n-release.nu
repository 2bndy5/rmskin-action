# This script automates the release process for all of the packages in this repository.
# In order, this script does the following:
#
# 1. Bump version number in Cargo.toml manifest.
#
#    This step requires `cargo-edit` (specifically `set-version` feature) installed.
#
# 2. Updates the CHANGELOG.md
#
#    Requires `uv` installed to install/run `git-cliff` (see https://git-cliff.org).
#    `git-cliff` is used to regenerate the change logs from git history.
#
#    NOTE: `git cliff` uses GITHUB_TOKEN env var to access GitHub's REST API for
#    fetching certain data (like PR labels and commit author's username).
#
# 3. Pushes the changes from (steps 1 and 2) to remote
#
# 4. Creates a GitHub Release and uses the section from the CHANGELOG about the new tag
#    as a release description.
#
#    Requires `gh-cli` (see https://cli.github.com) to be installed to create the release
#    and push the tag.
#
#    NOTE: This step also tags the commit from step 3.
#    When a tag is pushed to the remote, the CI builds are triggered and
#    a package are published to crates.io
#
#    NOTE: In a CI run, the GITHUB_TOKEN env var to authenticate access.
#    Locally, you can use `gh login` to interactively authenticate the user account.


# Run an external command and output its elapsed time.
#
# Not useful if you need to capture the command's output.
export def --wrapped run-cmd [...cmd: string] {
    let app = if (
        ($cmd | first) == "git"
        or ($cmd | first) == "gh"
    ) {
        ($cmd | first 2) | str join " "
    } else if ($cmd | first) == 'uvx' {
        $cmd | skip 1 | first
    } else {
        ($cmd | first)
    }
    print $"(ansi blue)\nRunning(ansi reset) ($cmd | str join ' ')"
    let elapsed = timeit {|| ^($cmd | first) ...($cmd | skip 1)}
    print $"(ansi magenta)($app) took ($elapsed)(ansi reset)"
}

const GIT_CLIFF_CONFIG = [--config .config/cliff.toml]

# Bump the version.
#
# This function also updates known occurrences of the old version spec to
# the new version spec in various places (like README.md and action.yml).
export def bump-version [
    component?: string # The version component (major, minor, patch) to bump. If not given, `git-cliff` will guess the next version based on unreleased git history.
    --dry-run, # Prevent this function from making changes to disk/files.
] {
    mut args = if ($component | is-empty) {
        let ver = (
            (^uvx git-cliff ...$GIT_CLIFF_CONFIG --bumped-version)
            | str trim
            | str trim --left --char "v"
        )
        let old = open "Cargo.toml" | get "package" | "version"
        print $"git-cliff predicts the next version to be ($ver)"
        [$ver]
    } else {
        [--bump $component]
    }
    if $dry_run {
        $args = $args | append "--dry-run"
    }
    let output = (^cargo set-version ...$args) | complete
    let result = if (($output | get exit_code) == 0) {
        (
            $output
            | get stderr
            | lines
            | each { $in | str trim }
            | where { $in | str starts-with "Upgrading "}
            | first
            | parse "Upgrading {pkg} from {old} to {new}"
            | first
        )
    } else {
        error make {msg: ($output | get stderr)}
    }
    print $"bumped ($result | get old) to ($result | get new)"
    if not $dry_run {
        # update the version in various places
        (
            open action.yml --raw
            | str replace $"STANDALONE_BIN_VER: '($result | get old)'" $"STANDALONE_BIN_VER: '($result | get new)'"
            | save --force action.yml
        )
        print "Updated action.yml"
        (
            open README.md
            | str replace $"rmskin-action@v($result | get old)" $"rmskin-action@v($result | get new)"
            | save --force README.md
        )
        print "Updated README.md"
    }
    $result | get new
}

export const RELEASE_NOTES = $nu.temp-path | path join "ReleaseNotes.md"
const CHANGELOG = "CHANGELOG.md"

# Use `git-cliff` tp generate changes.
#
# If `--unreleased` is asserted, then the `git-cliff` output will be saved to `$RELEASE_NOTES`.
# Otherwise, the generated changes will span the entire git history and be saved to CHANGELOG.md.
export def gen-changes [
    tag: string, # the new version tag to use for unreleased changes.
    --unreleased, # only generate changes from unreleased version.
] {
    mut args = $GIT_CLIFF_CONFIG | append [--tag, $tag]
    let prompt = if $unreleased {
        let out_path = $RELEASE_NOTES
        $args = $args | append [--strip, header, --unreleased, --output, $out_path]
        {out_path: $out_path, log_prefix: "Generated"}
    } else {
        let out_path = $CHANGELOG
        $args = $args | append [--output, $out_path]
        {out_path: $out_path, log_prefix: "Updated"}
    }
    run-cmd uvx git-cliff ...$args
    print ($prompt | format pattern "{log_prefix} {out_path}")
}

# Move applicable rolling tags to the checked out HEAD.
#
# For example, `v1` and `v1.2` are moved to the newer `v1.2.3` ref.
export def mv-rolling-tags [
    ver: string # The fully qualified version of the new tag (without `v` prefixed).
] {
    let tags = ^git tag --list | lines
    let tag = $ver | parse "{major}.{minor}.{patch}" | first
    let major_tag = $"v($tag | get major)"
    let minor_tag = $"v($tag | get major).($tag | get minor)"
    for t in [$major_tag, $minor_tag] {
        if ($t in $tags) {
            # delete local tag
            run-cmd git tag -d $t
            # delete remote tags
            run-cmd git push origin $":refs/tags/($t)"
        }
        run-cmd git tag $t
        run-cmd git push origin $t
        print $"Adjusted tags ($t)"
    }
}

# Is the the default branch currently checked out?
export def is-on-main [] {
    let branch = (
        ^git branch
        | lines
        | where {$in | str starts-with '*'}
        | first
        | str trim --left --char '*'
        | str trim
    ) == "main"
    $branch
}

# The main function of this script.
#
# The acceptable `component` values are what `cargo set-version --bump` accepts:
#
# - manor
# - minor
# - patch
export def main [
    component?: string, # If not provided, `git-cliff` will guess the next version based on unreleased git history.
] {
    let is_main = is-on-main
    let ver = if not $is_main {
        bump-version $component --dry-run
    } else {
        bump-version $component
    }
    let tag = $"v($ver)"
    if not $is_main {
        gen-changes $tag --unreleased
        open $RELEASE_NOTES | print
    } else {
        gen-changes $tag
        gen-changes $tag --unreleased
    }
    let is_ci = $env | get --optional CI | into bool --relaxed
    if not $is_main {
        let prompt = "Not checked out on default branch!"
        if ($is_ci) {
            print $"::error::($prompt)"
        } else {
            print $"(ansi yellow)($prompt)(ansi reset)"
        }
        exit 1
    }
    if ($is_ci) {
        run-cmd git config --global user.name $"($env.GITHUB_ACTOR)"
        run-cmd git config --global user.email $"($env.GITHUB_ACTOR_ID)+($env.GITHUB_ACTOR)@users.noreply.github.com"
    }
    run-cmd git add --all
    run-cmd git commit -m $"build: bump version to ($tag)"
    run-cmd git push

    mv-rolling-tags $ver

    print $"Deploying ($tag)"
    run-cmd gh release create $tag --notes-file $RELEASE_NOTES
}
