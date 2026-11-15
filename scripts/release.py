#!/usr/bin/env python3
"""Build and verify the immutable release contract (Python 3.11+, stdlib only).

Packaging and integrity validation are local. Only the explicit ``publish``
subcommand contacts GitHub, and it requires a matching pushed-tag event.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import tomllib

REPOSITORY = "veryshyjelly/craftcoders-cli"
PLATFORMS = ("darwin_amd64", "darwin_arm64", "linux_amd64", "linux_arm64")
ENTRIES = {"craftcoders": 0o755, "README.md": 0o644, "LICENSE": 0o644}
STABLE_VERSION = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
ROOT = Path(__file__).resolve().parents[1]


def package_version(root=ROOT):
    with (root / "Cargo.toml").open("rb") as source:
        package = tomllib.load(source)["package"]
    if package["name"] != "craftcoders":
        raise ValueError("unexpected Cargo package name")
    version = package["version"]
    if not re.fullmatch(STABLE_VERSION, version):
        raise ValueError("Cargo version must be stable MAJOR.MINOR.PATCH")
    return version


def validate_tag(tag, version):
    if not re.fullmatch("v" + STABLE_VERSION, tag) or tag != "v" + version:
        raise ValueError(f"expected stable tag v{version}, got {tag!r}")
    return version


def archive_names(version):
    if not re.fullmatch(STABLE_VERSION, version):
        raise ValueError("invalid version")
    return tuple(f"craftcoders_{version}_{platform}.tar.gz" for platform in PLATFORMS)


def smoke(binary, version):
    binary = Path(binary).resolve(strict=True)
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError("binary must be an executable regular file")
    result = subprocess.run([str(binary), "--version"], capture_output=True, text=True, check=True)
    if result.stdout != f"craftcoders {version}\n" or result.stderr:
        raise ValueError(f"unexpected --version output: {result.stdout!r}, {result.stderr!r}")


def validate_archive(path):
    with tarfile.open(path, "r:gz") as archive:
        members = archive.getmembers()
        if len(members) != len(ENTRIES) or {member.name for member in members} != set(ENTRIES):
            raise ValueError(f"{path}: expected exactly root craftcoders, README.md, LICENSE")
        for member in members:
            if not member.isfile() or member.mode != ENTRIES[member.name] or member.size <= 0:
                raise ValueError(f"{path}: invalid type, permissions, or size for {member.name}")


def package(binary, platform, output, version, root=ROOT):
    if platform not in PLATFORMS:
        raise ValueError("unsupported platform")
    smoke(binary, version)
    sources = {"craftcoders": Path(binary), "README.md": root / "README.md", "LICENSE": root / "LICENSE"}
    for source in sources.values():
        if not source.is_file():
            raise ValueError(f"missing package input: {source}")
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"craftcoders_{version}_{platform}.tar.gz"
    # Explicit metadata avoids nested paths, symlinks, or source permission leaks.
    with tarfile.open(destination, "w:gz", format=tarfile.USTAR_FORMAT) as archive:
        for name, mode in ENTRIES.items():
            info = tarfile.TarInfo(name)
            info.size = sources[name].stat().st_size
            info.mode = mode
            with sources[name].open("rb") as source:
                archive.addfile(info, source)
    validate_archive(destination)
    return destination


def sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def validate_outputs(directory, version):
    expected = set(archive_names(version))
    actual = {entry.name for entry in directory.iterdir()}
    if actual - {"checksums.txt"} != expected:
        raise ValueError(f"expected exactly four platform archives, got {sorted(actual)}")
    for name in sorted(expected):
        path = directory / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"invalid archive file: {name}")
        validate_archive(path)
    return {name: sha256(directory / name) for name in sorted(expected)}


def manifest_text(checksums):
    return "".join(f"{checksums[name]}  {name}\n" for name in sorted(checksums))


def write_manifest(directory, version):
    checksums = validate_outputs(directory, version)
    (directory / "checksums.txt").write_text(manifest_text(checksums), encoding="utf-8")
    return checksums


def validate_release(directory, version):
    checksums = validate_outputs(directory, version)
    manifest = directory / "checksums.txt"
    if manifest.is_symlink() or not manifest.is_file():
        raise ValueError("missing or invalid checksums.txt")
    # Exact comparison enforces ordering, lowercase, separators, and final newline.
    if manifest.read_bytes() != manifest_text(checksums).encode("utf-8"):
        raise ValueError("checksums.txt is malformed, incomplete, or does not match archives")
    return checksums


def require_tag_event(tag, environ=None):
    environ = os.environ if environ is None else environ
    if (environ.get("GITHUB_EVENT_NAME"), environ.get("GITHUB_REF_TYPE"), environ.get("GITHUB_REF_NAME"), environ.get("GITHUB_REPOSITORY")) != ("push", "tag", tag, REPOSITORY):
        raise ValueError("publication requires a pushed tag in the upstream CLI repository")


def publish(directory, tag, version, run=subprocess.run):
    """Create a new draft, verify its remote bytes, then publish; never overwrite.

    Any error after create deliberately leaves a draft for explicit recovery.
    ``run`` is injectable so unit tests never invoke GitHub or a network.
    """
    validate_tag(tag, version)
    checksums = validate_release(directory, version)

    def gh(*args, check=True):
        return run(["gh", *args], capture_output=True, text=True, check=check)

    # The REST tag endpoint cannot find drafts. An authenticated complete list
    # includes drafts visible to our contents-write token; all lookup errors fail.
    pages = json.loads(gh("api", "--paginate", "--slurp",
                         f"repos/{REPOSITORY}/releases?per_page=100").stdout)
    if (not isinstance(pages, list) or not pages
            or any(not isinstance(page, list) for page in pages)
            or any(not isinstance(item, dict) or not isinstance(item.get("tag_name"), str)
                   for page in pages for item in page)):
        raise ValueError("invalid release listing; cannot establish release absence")
    if any(item["tag_name"] == tag for page in pages for item in page):
        raise ValueError("release or draft already exists; inspect it and recover explicitly; never overwrite a published release")

    files = [directory / name for name in (*archive_names(version), "checksums.txt")]
    gh("release", "create", tag, "--repo", REPOSITORY, "--verify-tag", "--draft",
       "--title", tag, "--notes", f"CraftCoders {version}. See README.md for platform baselines and installation.",
       *map(str, files))
    # gh release view performs draft-aware GraphQL lookup, then REST by ID.
    # Use its documented databaseId field to fetch raw REST asset metadata.
    identity = json.loads(gh("release", "view", tag, "--repo", REPOSITORY,
                             "--json", "databaseId").stdout)
    release_id = identity.get("databaseId")
    if type(release_id) is not int or release_id <= 0:
        raise ValueError("invalid remote draft release ID; leaving draft unpublished")
    release = json.loads(gh("api", f"repos/{REPOSITORY}/releases/{release_id}").stdout)
    assets = release.get("assets", [])
    expected = {path.name for path in files}
    if (release.get("tag_name") != tag or release.get("draft") is not True
            or release.get("prerelease") is not False
            or len(assets) != len(expected) or {asset.get("name") for asset in assets} != expected):
        raise ValueError("remote draft identity or asset set is incomplete; leaving draft unpublished")
    for asset in assets:
        if asset.get("state") != "uploaded" or asset.get("size") != (directory / asset["name"]).stat().st_size:
            raise ValueError("remote draft asset upload is incomplete; leaving draft unpublished")
    with tempfile.TemporaryDirectory(prefix="craftcoders-remote-") as temporary:
        downloaded = Path(temporary)
        gh("release", "download", tag, "--repo", REPOSITORY, "--dir", str(downloaded))
        if validate_release(downloaded, version) != checksums:
            raise ValueError("remote draft bytes differ; leaving draft unpublished")
        if (downloaded / "checksums.txt").read_bytes() != (directory / "checksums.txt").read_bytes():
            raise ValueError("remote draft manifest differs; leaving draft unpublished")
    gh("release", "edit", tag, "--repo", REPOSITORY, "--draft=false")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    for name in ("version", "package", "manifest", "validate", "publish"):
        command = subparsers.add_parser(name)
        command.add_argument("--tag", required=name == "publish", default="")
        if name in ("manifest", "validate", "publish"):
            command.add_argument("--directory", type=Path, required=True)
        if name == "package":
            command.add_argument("--binary", type=Path, required=True)
            command.add_argument("--platform", choices=PLATFORMS, required=True)
            command.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    version = package_version()
    if args.tag:
        validate_tag(args.tag, version)
    if args.command == "version":
        print(version)
    elif args.command == "package":
        print(package(args.binary, args.platform, args.output, version))
    elif args.command == "manifest":
        write_manifest(args.directory, version)
    elif args.command == "validate":
        validate_release(args.directory, version)
    elif args.command == "publish":
        require_tag_event(args.tag)
        publish(args.directory, args.tag, version)


if __name__ == "__main__":
    main()
