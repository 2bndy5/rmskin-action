# This script automates the release process for all of the packages in this repository.
# In order, this script does the following:
#
# 1. Bump version number in Cargo.toml manifest.
#
#    This step requires `cargo-edit` installed.
#
# 2. Updates the CHANGELOG.md
#
#    Requires `git-cliff` (see https://git-cliff.org) to be installed
#    to regenerate the change logs from git history.
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


let IN_CI = $env | get --ignore-errors CI | default "false" | ($in == "true") or ($in == true)

# Bump the version per the given component name (major, minor, patch)
#
# This function also updates known occurrences of the old version spec to
# the new version spec in various places (like README.md and action.yml).
def bump-version [
    component: string # the version component to bump
] {
    mut args = [--bump $component]
    if (not $IN_CI) {
        $args = $args | append "--dry-run"
    }
    let result = (
        cargo set-version ...$args e>| lines
        | first
        | str trim
        | parse "Upgrading {pkg} from {old} to {new}"
        | first
    )
    print $"bumped ($result | get old) to ($result | get new)"
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
    $result | get new
}

# Use `git-cliff` tp generate changes.
#
# If `--unreleased` is asserted, then the `git-cliff` output will be saved to .config/ReleaseNotes.md.
# Otherwise, the generated changes will span the entire git history and be saved to CHANGELOG.md.
def gen-changes [
    tag: string, # the new version tag to use for unreleased changes.
    --unreleased, # only generate changes from unreleased version.
] {
    mut args = [--tag, $tag, --config, .config/cliff.toml]
    let prompt = if $unreleased {
        let out_path = ".config/ReleaseNotes.md"
        $args = $args | append [--strip, header, --unreleased, --output, $out_path]
        {out_path: $out_path, log_prefix: "Generated"}
    } else {
        let out_path = "CHANGELOG.md"
        $args = $args | append [--output, $out_path]
        {out_path: $out_path, log_prefix: "Updated"}
    }
    ^git-cliff ...$args
    print ($prompt | format pattern "{log_prefix} {out_path}")
}

# Move applicable rolling tags to the checked out HEAD.
#
# For example, `v1` and `v1.2` are moved to the newer `v1.2.3` ref.
def mv-rolling-tags [
    ver: string # The fully qualified version of the new tag (without `v` prefixed).
] {
    let tags = ^git tag --list | lines
    let tag = $ver | parse "{major}.{minor}.{patch}" | first
    let major_tag = $"v($tag | get major)"
    let minor_tag = $"v($tag | get major).($tag | get minor)"
    for t in [$major_tag, $minor_tag] {
        if ($t in $tags) {
            # delete local tag
            git tag -d $t
            # delete remote tags
            git push origin $":refs/tags/($t)"
        }
        git tag $t
        git push origin $t
        print $"Adjusted tags ($t)"
    }
}

# Is the the default branch currently checked out?
def is-on-main [] {
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

# Publish this package to crates.io
#
# This requires a token in $env.CARGO_REGISTRY_TOKEN for authentication.
def deploy-crate [] {
    ^cargo publish
}

# Publish a GitHub Release for the given tag.
#
# This requires a token in $env.GITHUB_TOKEN for authentication.
def gh-release [tag: string] {
    ^gh release create $tag --notes-file ".config/ReleaseNotes.md"
}

# The main function of this script.
#
# The `component` parameter is a required CLI option:
#     nu .github/workflows/bump-n-release.nu patch
#
# The acceptable `component` values are what `cargo set-version` accepts:
#
# - manor
# - minor
# - patch
def main [component: string] {
    let ver = bump-version $component
    let tag = $"v($ver)"
    gen-changes $tag
    gen-changes $tag --unreleased
    let is_main = is-on-main
    if not $is_main {
        print $"(ansi yellow)Not checked out on default branch!(ansi reset)"
    }
    if $IN_CI and $is_main {
        git config --global user.name $"($env.GITHUB_ACTOR)"
        git config --global user.email $"($env.GITHUB_ACTOR_ID)+($env.GITHUB_ACTOR)@users.noreply.github.com"
        git add --all
        git commit -m $"build: bump version to ($tag)"
        git push
        mv-rolling-tags $ver
        print "Publishing crate"
        deploy-crate
        print $"Deploying ($tag)"
        gh-release $tag
    } else if $is_main {
        print $"(ansi yellow)Not deploying from local clone.(ansi reset)"
    }
}
