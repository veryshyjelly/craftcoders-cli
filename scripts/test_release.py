"""Local release contract tests. All GitHub commands are mocked, never executed."""

import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest

import release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "dist"
        self.binary = self.root / "craftcoders"
        self.binary.write_text('#!/bin/sh\nprintf "craftcoders 0.1.0\\n"\n', encoding="utf-8")
        self.binary.chmod(0o755)
        for name in ("README.md", "LICENSE"):
            (self.root / name).write_text(name + " contents\n", encoding="utf-8")

    def package(self, platform="darwin_arm64"):
        return release.package(self.binary, platform, self.output, "0.1.0", self.root)

    def complete(self):
        for platform in release.PLATFORMS:
            self.package(platform)
        release.write_manifest(self.output, "0.1.0")

    def archive(self, entries):
        self.output.mkdir(exist_ok=True)
        destination = self.output / release.archive_names("0.1.0")[0]
        with tarfile.open(destination, "w:gz") as archive:
            for name, mode, kind in entries:
                member = tarfile.TarInfo(name)
                member.mode = mode
                member.type = kind
                member.size = 4 if kind == tarfile.REGTYPE else 0
                member.linkname = "elsewhere" if kind == tarfile.SYMTYPE else ""
                archive.addfile(member, io.BytesIO(b"data") if member.size else None)
        return destination

    def fake_gh(self, *, existing=False, lookup_failure=False, missing=False, corrupt=False, upload_failure=False, duplicate=False, view_failure=False, metadata_failure=False, malformed_pages=False, invalid_id=False):
        self.calls = []
        self.tag_endpoint_404s = 0

        def run(command, **kwargs):
            self.calls.append(command)
            self.assertTrue(kwargs["capture_output"])
            args = command[1:]
            if args[0] == "api":
                endpoint = args[-1]
                # GitHub's REST tag endpoint returns 404 for drafts, including
                # newly created ones. Keep that real behavior in the fixture.
                if "/releases/tags/" in endpoint:
                    self.tag_endpoint_404s += 1
                    if kwargs["check"]:
                        raise subprocess.CalledProcessError(1, command, stderr="gh: Not Found (HTTP 404)")
                    return subprocess.CompletedProcess(command, 1, stdout="", stderr="gh: Not Found (HTTP 404)")
                if endpoint.endswith("releases?per_page=100"):
                    self.assertIn("--paginate", args)
                    self.assertIn("--slurp", args)
                    if lookup_failure:
                        raise subprocess.CalledProcessError(1, command, stderr=f"HTTP {lookup_failure}")
                    # Matching release is deliberately on a later page. Drafts
                    # and published releases are both visible in this listing.
                    pages = [[{"tag_name": "v0.0.9", "draft": False}],
                             [{"tag_name": "v0.1.0", "draft": existing == "draft"}]] if existing else [[]]
                    return subprocess.CompletedProcess(command, 0, stdout=json.dumps({"bad": []} if malformed_pages else pages), stderr="")
                self.assertEqual(endpoint, f"repos/{release.REPOSITORY}/releases/73")
                if metadata_failure:
                    raise subprocess.CalledProcessError(1, command, stderr="HTTP 503")
                assets = [{"name": path.name, "size": path.stat().st_size, "state": "uploaded"} for path in self.output.iterdir()]
                if missing:
                    assets.pop()
                if duplicate:
                    assets[-1] = assets[0]
                return subprocess.CompletedProcess(command, 0, stdout=json.dumps({"tag_name": "v0.1.0", "draft": True, "prerelease": False, "assets": assets}), stderr="")
            if args[:2] == ["release", "view"]:
                self.assertEqual(args, ["release", "view", "v0.1.0", "--repo", release.REPOSITORY, "--json", "databaseId"])
                if view_failure:
                    raise subprocess.CalledProcessError(1, command, stderr="HTTP 403")
                return subprocess.CompletedProcess(command, 0, stdout=json.dumps({"databaseId": "unsafe" if invalid_id else 73}), stderr="")
            if args[:2] == ["release", "create"] and upload_failure:
                raise subprocess.CalledProcessError(1, command, stderr="upload failed")
            if args[:2] == ["release", "download"]:
                destination = Path(args[args.index("--dir") + 1])
                for path in self.output.iterdir():
                    shutil.copyfile(path, destination / path.name)
                if corrupt:
                    (destination / "checksums.txt").write_text("corrupt\n", encoding="utf-8")
            return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

        return run

    def assert_not_published(self):
        self.assertFalse(any(command[1:3] == ["release", "edit"] for command in self.calls))

    def test_stable_tag_must_match_package_version(self):
        self.assertEqual(release.validate_tag("v0.1.0", "0.1.0"), "0.1.0")
        for tag in ("0.1.0", "v0.1.1", "v01.1.0", "v0.1.0-rc1", "v0.1.0+build", "v0.1.0/../x", "v0.1.0\n", "$(echo dangerous)"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate_tag(tag, "0.1.0")

    def test_cargo_metadata_identity(self):
        (self.root / "Cargo.toml").write_text('[package]\nname="craftcoders"\nversion="0.1.0"\n', encoding="utf-8")
        self.assertEqual(release.package_version(self.root), "0.1.0")
        for text in ('name="other"\nversion="0.1.0"', 'name="craftcoders"\nversion="0.1.0-rc1"'):
            (self.root / "Cargo.toml").write_text("[package]\n" + text, encoding="utf-8")
            with self.assertRaises(ValueError):
                release.package_version(self.root)

    def test_archive_names_are_exact_and_sorted(self):
        self.assertEqual(release.archive_names("0.1.0"), (
            "craftcoders_0.1.0_darwin_amd64.tar.gz", "craftcoders_0.1.0_darwin_arm64.tar.gz",
            "craftcoders_0.1.0_linux_amd64.tar.gz", "craftcoders_0.1.0_linux_arm64.tar.gz"))
        with self.assertRaises(ValueError):
            release.archive_names("../evil")

    def test_package_smokes_binary_and_normalizes_root_entries_and_modes(self):
        destination = self.package()
        self.assertEqual(destination.name, "craftcoders_0.1.0_darwin_arm64.tar.gz")
        with tarfile.open(destination) as archive:
            self.assertEqual({member.name: member.mode for member in archive.getmembers()}, release.ENTRIES)
            self.assertTrue(all(member.isfile() for member in archive.getmembers()))
            self.assertEqual(archive.extractfile("craftcoders").read(), self.binary.read_bytes())
        release.validate_archive(destination)

    def test_smoke_rejects_wrong_or_extra_output_and_failed_binary(self):
        for body in ('printf "craftcoders 9.0.0\\n"', 'printf "craftcoders 0.1.0\\nextra\\n"', 'printf "craftcoders 0.1.0\\n"; echo warning >&2', 'exit 3'):
            self.binary.write_text("#!/bin/sh\n" + body + "\n", encoding="utf-8")
            with self.subTest(body=body), self.assertRaises((ValueError, subprocess.CalledProcessError)):
                self.package()
        self.assertFalse(self.output.exists())

    def test_missing_non_executable_binary_and_missing_docs_are_rejected(self):
        self.binary.chmod(0o644)
        with self.assertRaises(ValueError):
            self.package()
        self.binary.chmod(0o755)
        (self.root / "LICENSE").unlink()
        with self.assertRaises(ValueError):
            self.package()
        self.binary.unlink()
        with self.assertRaises(FileNotFoundError):
            self.package()

    def test_unknown_platform_is_rejected(self):
        with self.assertRaises(ValueError):
            self.package("windows_arm64")

    def test_archive_rejects_paths_duplicates_links_wrong_modes_and_missing_entries(self):
        valid = [(name, mode, tarfile.REGTYPE) for name, mode in release.ENTRIES.items()]
        variants = [valid[:-1], valid + [valid[0]], valid + [("extra", 0o644, tarfile.REGTYPE)],
                    [("../craftcoders", 0o755, tarfile.REGTYPE), *valid[1:]],
                    [("craftcoders", 0o644, tarfile.REGTYPE), *valid[1:]],
                    [("craftcoders", 0o4755, tarfile.REGTYPE), *valid[1:]],
                    [("craftcoders", 0o755, tarfile.SYMTYPE), *valid[1:]]]
        for entries in variants:
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                release.validate_archive(self.archive(entries))

    def test_manifest_exactly_covers_four_outputs_with_hashes(self):
        self.complete()
        checksums = release.validate_release(self.output, "0.1.0")
        self.assertEqual(tuple(checksums), release.archive_names("0.1.0"))
        text = (self.output / "checksums.txt").read_text(encoding="utf-8")
        self.assertTrue(text.endswith("\n"))
        self.assertEqual(len(text.splitlines()), 4)
        for line in text.splitlines():
            self.assertRegex(line, r"^[a-f0-9]{64}  craftcoders_0\.1\.0_(darwin|linux)_(amd64|arm64)\.tar\.gz$")
            digest, name = line.split("  ")
            self.assertEqual(digest, release.sha256(self.output / name))

    def test_missing_or_unexpected_matrix_outputs_are_rejected(self):
        self.package()
        with self.assertRaises(ValueError):
            release.write_manifest(self.output, "0.1.0")
        self.complete()
        (self.output / "unexpected.tar.gz").write_bytes(b"unexpected")
        with self.assertRaises(ValueError):
            release.validate_release(self.output, "0.1.0")

    def test_archive_symlink_is_rejected(self):
        self.complete()
        path = self.output / release.archive_names("0.1.0")[0]
        renamed = self.root / "external.tar.gz"
        path.rename(renamed)
        path.symlink_to(renamed)
        with self.assertRaises(ValueError):
            release.validate_release(self.output, "0.1.0")

    def test_missing_manifest_and_invalid_manifest_forms_are_rejected(self):
        self.complete()
        manifest = self.output / "checksums.txt"
        original = manifest.read_text(encoding="utf-8")
        manifest.unlink()
        with self.assertRaises(ValueError):
            release.validate_release(self.output, "0.1.0")
        for invalid in (original.rstrip("\n"), original.upper(), original.replace("  ", " *"), original + original.splitlines()[0] + "\n", "bad\n", "\n".join(reversed(original.splitlines())) + "\n"):
            manifest.write_text(invalid, encoding="utf-8")
            with self.subTest(invalid=invalid[:80]), self.assertRaises(ValueError):
                release.validate_release(self.output, "0.1.0")

    def test_changed_archive_is_rejected_by_manifest(self):
        self.complete()
        self.binary.write_text('#!/bin/sh\n# changed bytes\nprintf "craftcoders 0.1.0\\n"\n', encoding="utf-8")
        self.package()
        with self.assertRaises(ValueError):
            release.validate_release(self.output, "0.1.0")

    def test_publication_requires_upstream_pushed_tag(self):
        expected = {"GITHUB_EVENT_NAME": "push", "GITHUB_REF_TYPE": "tag", "GITHUB_REF_NAME": "v0.1.0", "GITHUB_REPOSITORY": release.REPOSITORY}
        release.require_tag_event("v0.1.0", expected)
        for key, value in (("GITHUB_EVENT_NAME", "workflow_dispatch"), ("GITHUB_EVENT_NAME", "pull_request"), ("GITHUB_REF_TYPE", "branch"), ("GITHUB_REF_NAME", "v0.1.1"), ("GITHUB_REPOSITORY", "someone/fork")):
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                release.require_tag_event("v0.1.0", {**expected, key: value})

    def test_publish_uploads_five_assets_verifies_remote_bytes_then_edits(self):
        self.complete()
        release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh())
        create = next(command for command in self.calls if command[1:3] == ["release", "create"])
        self.assertIn("--verify-tag", create)
        self.assertIn("--draft", create)
        self.assertEqual({Path(arg).name for arg in create[-5:]}, {*release.archive_names("0.1.0"), "checksums.txt"})
        self.assertEqual(self.calls[-1], ["gh", "release", "edit", "v0.1.0", "--repo", release.REPOSITORY, "--draft=false"])
        self.assertEqual(self.calls[-2][1:3], ["release", "download"])
        self.assertEqual(self.calls[2][1:3], ["release", "view"])
        self.assertEqual(self.calls[3], ["gh", "api", f"repos/{release.REPOSITORY}/releases/73"])
        self.assertEqual(self.tag_endpoint_404s, 0)

    def test_existing_published_release_or_draft_on_later_page_is_never_overwritten(self):
        self.complete()
        for state in ("published", "draft"):
            with self.subTest(state=state), self.assertRaises(ValueError):
                release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh(existing=state))
            self.assertEqual(len(self.calls), 1)
            self.assertEqual(self.tag_endpoint_404s, 0)

    def test_lookup_auth_rate_limit_not_found_and_server_failures_never_create_release(self):
        self.complete()
        for status in (401, 403, 404, 429, 503):
            with self.subTest(status=status), self.assertRaises(subprocess.CalledProcessError):
                release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh(lookup_failure=status))
            self.assertEqual(len(self.calls), 1)
            self.assert_not_published()

    def test_malformed_release_listing_does_not_establish_absence(self):
        self.complete()
        with self.assertRaises(ValueError):
            release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh(malformed_pages=True))
        self.assertEqual(len(self.calls), 1)

    def test_new_draft_view_or_metadata_lookup_failure_never_publishes(self):
        self.complete()
        for failure in ("view_failure", "metadata_failure", "invalid_id"):
            with self.subTest(failure=failure), self.assertRaises((ValueError, subprocess.CalledProcessError)):
                release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh(**{failure: True}))
            self.assert_not_published()

    def test_upload_failure_missing_duplicate_or_corrupt_remote_assets_never_publish(self):
        self.complete()
        for failure in ("upload_failure", "missing", "duplicate", "corrupt"):
            with self.subTest(failure=failure), self.assertRaises((ValueError, subprocess.CalledProcessError)):
                release.publish(self.output, "v0.1.0", "0.1.0", self.fake_gh(**{failure: True}))
            self.assert_not_published()

    def test_bad_local_assets_or_tag_prevent_any_remote_command(self):
        self.complete()
        runner = self.fake_gh()
        with self.assertRaises(ValueError):
            release.publish(self.output, "v0.1.1", "0.1.0", runner)
        (self.output / "checksums.txt").write_text("invalid\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            release.publish(self.output, "v0.1.0", "0.1.0", runner)
        self.assertEqual(self.calls, [])


if __name__ == "__main__":
    unittest.main()
