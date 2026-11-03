"""Tests of release.py. Run: python3 -I -m unittest discover -s scripts/release -p 'test_*.py'

Criteria: the table "Acceptance criteria and their tests" in the spec
docs/superpowers/specs/2026-10-14-STORY-48-install-jag-from-github-releases-crates-io-or-npm-design.md.
"""

import contextlib
import gzip
import hashlib
import io
import json
import os
import re
import shutil
import stat
import subprocess
import tarfile
import tempfile
import tomllib
import unittest
import urllib.error
from pathlib import Path

import release

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"
DEPS_WORKFLOW = ROOT / ".github" / "workflows" / "deps.yml"
VERSION = "1.2.3"


def write(path, text="", mode=0o644):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(text, bytes):
        path.write_bytes(text)
    else:
        path.write_text(text, encoding="utf-8")
    os.chmod(path, mode)
    return path


def make_tgz(path, members):
    """members: a list of (name, bytes or None for a folder, mode)."""
    with tarfile.open(path, "w:gz") as tar:
        for name, data, mode in members:
            info = tarfile.TarInfo(name)
            info.mode = mode
            if data is None:
                info.type = tarfile.DIRTYPE
                tar.addfile(info)
            else:
                info.size = len(data)
                tar.addfile(info, io.BytesIO(data))
    return path


def run_main(argv):
    err = io.StringIO()
    out = io.StringIO()
    with contextlib.redirect_stderr(err), contextlib.redirect_stdout(out):
        code = release.main(argv)
    return code, out.getvalue(), err.getvalue()


class TempCase(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="jag-release-test-"))
        self.addCleanup(shutil.rmtree, self.tmp, True)


class VersionTests(TempCase):
    def cargo(self, version, lock_version=None):
        toml = write(self.tmp / "Cargo.toml", f'[package]\nname = "jag-cli"\nversion = "{version}"\n')
        lock = write(
            self.tmp / "Cargo.lock",
            f'version = 4\n\n[[package]]\nname = "jag-cli"\nversion = "{lock_version or version}"\n',
        )
        return ["--cargo-toml", str(toml), "--cargo-lock", str(lock)]

    def test_a_tag_that_agrees(self):
        code, out, _ = run_main(["version", *self.cargo(VERSION), "--tag", f"v{VERSION}"])
        self.assertEqual((code, out), (0, f"{VERSION}\n"))

    def test_no_tag_prints_the_version(self):
        code, out, _ = run_main(["version", *self.cargo(VERSION)])
        self.assertEqual((code, out), (0, f"{VERSION}\n"))

    def test_a_tag_that_does_not_agree(self):
        code, _, err = run_main(["version", *self.cargo("1.2.2"), "--tag", "v1.2.3"])
        self.assertEqual(code, 1)
        self.assertEqual(err, "release: the tag v1.2.3 does not match the version 1.2.2 in Cargo.toml.\n")

    def test_a_tag_with_no_v(self):
        code, _, _ = run_main(["version", *self.cargo(VERSION), "--tag", VERSION])
        self.assertEqual(code, 1)

    def test_a_pre_release_version_stops_the_release(self):
        for bad in ["1.2.3-rc.1", "1.2", "01.2.3", "1.2.3+build"]:
            code, _, err = run_main(["version", *self.cargo(bad)])
            self.assertEqual(code, 1, bad)
            self.assertIn("MAJOR.MINOR.PATCH", err)

    def test_cargo_lock_must_agree(self):
        code, _, err = run_main(["version", *self.cargo("1.2.3", lock_version="1.2.2")])
        self.assertEqual(code, 1)
        self.assertIn("Cargo.lock", err)

    def test_the_repository_agrees_with_itself(self):
        version = release.cargo_version(ROOT / "Cargo.toml", ROOT / "Cargo.lock")
        self.assertRegex(version, r"^\d+\.\d+\.\d+$")


class PlatformTests(unittest.TestCase):
    def test_the_matrix_has_the_four_targets(self):
        code, out, _ = run_main(["matrix"])
        self.assertEqual(code, 0)
        include = json.loads(out)["include"]
        self.assertEqual(
            [i["target"] for i in include],
            [
                "x86_64-unknown-linux-musl",
                "aarch64-unknown-linux-musl",
                "aarch64-apple-darwin",
                "x86_64-apple-darwin",
            ],
        )
        self.assertEqual(
            [i["runner"] for i in include],
            ["ubuntu-24.04", "ubuntu-24.04-arm", "macos-15", "macos-15-intel"],
        )

    def test_the_package_json_names_the_same_four_packages(self):
        package = json.loads((ROOT / "npm" / "jag" / "package.json").read_text())
        names = sorted(release.npm_platform_name(p) for p in release.PLATFORMS)
        self.assertEqual(sorted(package["optionalDependencies"]), names)
        self.assertEqual(
            names,
            ["@airf/jag-darwin-arm64", "@airf/jag-darwin-x64", "@airf/jag-linux-arm64", "@airf/jag-linux-x64"],
        )

    def test_each_platform_has_the_os_and_cpu_words_of_node(self):
        pairs = sorted((p["os"], p["cpu"]) for p in release.PLATFORMS)
        self.assertEqual(pairs, [("darwin", "arm64"), ("darwin", "x64"), ("linux", "arm64"), ("linux", "x64")])


class BinaryCheckTests(unittest.TestCase):
    STATIC_HEADERS = "Program Headers:\n  Type  Offset\n  LOAD  0x0\n  GNU_STACK 0x0\n"
    DYNAMIC_HEADERS = "Program Headers:\n  PHDR  0x40\n  INTERP 0x318\n  LOAD  0x0\n"

    def test_a_static_binary_passes(self):
        self.assertFalse(release.parse_readelf_headers(self.STATIC_HEADERS))
        self.assertFalse(release.parse_readelf_dynamic("There is no dynamic section in this file.\n"))

    def test_an_interp_header_fails(self):
        self.assertTrue(release.parse_readelf_headers(self.DYNAMIC_HEADERS))

    def test_a_needed_entry_fails(self):
        text = "Dynamic section at offset 0x1 contains 2 entries:\n 0x01 (NEEDED)  Shared library: [libc.so.6]\n"
        self.assertTrue(release.parse_readelf_dynamic(text))

    def test_macos_libraries(self):
        text = (
            "jag:\n"
            "\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1351.0.0)\n"
            "\t/System/Library/Frameworks/Security.framework/Versions/A/Security (compatibility version 1.0.0)\n"
        )
        self.assertEqual(release.bad_macos_libraries(release.parse_otool_libraries(text)), [])

    def test_a_library_in_opt_homebrew_fails(self):
        text = "jag:\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)\n\t/opt/homebrew/lib/libx.dylib (compat)\n"
        self.assertEqual(
            release.bad_macos_libraries(release.parse_otool_libraries(text)),
            ["/opt/homebrew/lib/libx.dylib"],
        )

    @unittest.skipUnless(shutil.which("readelf") and shutil.which("sh"), "needs readelf")
    def test_a_script_is_not_a_static_binary_but_the_version_check_runs(self):
        # A shell script is not an ELF file, so readelf fails: the check stops with exit 1.
        with tempfile.TemporaryDirectory() as tmp:
            binary = write(Path(tmp) / "jag", "#!/bin/sh\necho jag 1.2.3\n", 0o755)
            code, _, err = run_main(
                ["check-binary", "--target", "x86_64-unknown-linux-musl", "--version", VERSION, str(binary)]
            )
            self.assertEqual(code, 1)
            self.assertTrue(err.startswith("release:"))


class ArchiveTests(TempCase):
    TARGET = "x86_64-unknown-linux-musl"

    def make(self, out, binary_bytes=b"\x7fELF-binary", mtime=1_700_000_000):
        binary = write(self.tmp / "in" / "jag", binary_bytes, 0o755)
        readme = write(self.tmp / "in" / "README.md", "readme\n")
        license_file = write(self.tmp / "in" / "LICENSE", "license\n")
        notices = write(self.tmp / "in" / "THIRD-PARTY-LICENSES", "THIRD-PARTY LICENSES\n")
        return release.make_archive(VERSION, self.TARGET, binary, readme, license_file, notices, out, mtime)

    def test_the_folder_the_four_files_and_their_modes(self):
        path, _ = self.make(self.tmp / "out")
        self.assertEqual(path.name, f"jag-{VERSION}-{self.TARGET}.tar.gz")
        with tarfile.open(path, "r:gz") as tar:
            infos = {i.name: i for i in tar}
        folder = f"jag-{VERSION}-{self.TARGET}"
        self.assertEqual(
            sorted(infos),
            sorted(
                [folder, f"{folder}/jag", f"{folder}/README.md", f"{folder}/LICENSE", f"{folder}/THIRD-PARTY-LICENSES"]
            ),
        )
        self.assertTrue(infos[folder].isdir())
        self.assertEqual(infos[folder].mode, 0o755)
        self.assertEqual(infos[f"{folder}/jag"].mode, 0o755)
        self.assertEqual(infos[f"{folder}/README.md"].mode, 0o644)
        self.assertEqual(infos[f"{folder}/LICENSE"].mode, 0o644)
        self.assertEqual(infos[f"{folder}/THIRD-PARTY-LICENSES"].mode, 0o644)
        for info in infos.values():
            self.assertEqual((info.uid, info.gid, info.uname, info.gname), (0, 0, "", ""))
            self.assertEqual(info.mtime, 1_700_000_000)

    def test_the_checksum_line_agrees_with_the_archive(self):
        path, checksum = self.make(self.tmp / "out")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        self.assertEqual(checksum.name, f"{path.name}.sha256")
        self.assertEqual(checksum.read_text(), f"{digest}  {path.name}\n")

    def test_the_same_input_gives_the_same_archive(self):
        first, _ = self.make(self.tmp / "one")
        second, _ = self.make(self.tmp / "two")
        self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_the_gzip_header_has_no_name_and_no_time(self):
        path, _ = self.make(self.tmp / "out")
        header = path.read_bytes()[:10]
        self.assertEqual(header[:2], b"\x1f\x8b")
        self.assertEqual(header[3], 0, "no FNAME flag")
        self.assertEqual(header[4:8], b"\x00\x00\x00\x00", "no time")

    @unittest.skipUnless(shutil.which("sha256sum") or shutil.which("shasum"), "needs a checksum tool")
    def test_the_checksum_tool_reads_the_file(self):
        path, _ = self.make(self.tmp / "out")
        cmd = ["sha256sum", "-c"] if shutil.which("sha256sum") else ["shasum", "-a", "256", "-c"]
        result = subprocess.run([*cmd, f"{path.name}.sha256"], cwd=path.parent, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_the_command(self):
        binary = write(self.tmp / "jag", b"x", 0o755)
        readme = write(self.tmp / "README.md", "r")
        license_file = write(self.tmp / "LICENSE", "l")
        notices = write(self.tmp / "THIRD-PARTY-LICENSES", "n")
        code, out, _ = run_main(
            [
                "archive", "--target", self.TARGET, "--version", VERSION, "--binary", str(binary),
                "--out", str(self.tmp / "dist"), "--readme", str(readme), "--license", str(license_file),
                "--notices", str(notices), "--mtime", "5",
            ]
        )
        self.assertEqual((code, out), (0, f"jag-{VERSION}-{self.TARGET}.tar.gz\n"))

    def test_the_command_needs_the_notice_file(self):
        binary = write(self.tmp / "jag", b"x", 0o755)
        with self.assertRaises(SystemExit):
            with contextlib.redirect_stderr(io.StringIO()):
                release.main(
                    ["archive", "--target", self.TARGET, "--version", VERSION, "--binary", str(binary),
                     "--out", str(self.tmp / "dist")]
                )

    def test_the_license_and_the_notices_are_in_the_archive_byte_for_byte(self):
        license_file = ROOT / "LICENSE"
        notices = write(self.tmp / "THIRD-PARTY-LICENSES", "THIRD-PARTY LICENSES\n\ntext é\n")
        binary = write(self.tmp / "jag", b"x", 0o755)
        readme = write(self.tmp / "README.md", "r")
        path, _ = release.make_archive(VERSION, self.TARGET, binary, readme, license_file, notices, self.tmp / "o", 5)
        folder = f"jag-{VERSION}-{self.TARGET}"
        with tarfile.open(path, "r:gz") as tar:
            self.assertEqual(tar.extractfile(f"{folder}/LICENSE").read(), license_file.read_bytes())
            self.assertEqual(tar.extractfile(f"{folder}/THIRD-PARTY-LICENSES").read(), notices.read_bytes())


class NpmTests(TempCase):
    def build(self, target="x86_64-unknown-linux-musl", version=VERSION):
        out = self.tmp / "out"
        binary = write(self.tmp / "bin" / "jag", b"\x7fELF", 0o755)
        license_file = write(self.tmp / "LICENSE", "license\n")
        notices = write(self.tmp / "THIRD-PARTY-LICENSES", "THIRD-PARTY LICENSES\n")
        code, _, err = run_main(
            [
                "npm-platform", "--target", target, "--version", version, "--binary", str(binary),
                "--out", str(out), "--readme", str(ROOT / "npm" / "platform" / "README.md"),
                "--license", str(license_file), "--notices", str(notices),
            ]
        )
        self.assertEqual(code, 0, err)
        code, _, err = run_main(
            [
                "npm-main", "--version", version, "--out", str(out), "--source", str(ROOT / "npm" / "jag"),
                "--readme", str(ROOT / "README.md"), "--license", str(license_file),
            ]
        )
        self.assertEqual(code, 0, err)
        return out

    def test_the_main_package_has_the_version_in_each_place(self):
        out = self.build()
        package = json.loads((out / "jag" / "package.json").read_text())
        self.assertEqual(package["version"], VERSION)
        self.assertEqual(
            package["optionalDependencies"],
            {
                "@airf/jag-darwin-arm64": VERSION,
                "@airf/jag-darwin-x64": VERSION,
                "@airf/jag-linux-arm64": VERSION,
                "@airf/jag-linux-x64": VERSION,
            },
        )
        self.assertEqual(package["bin"], {"jag": "bin/jag.js"})
        self.assertEqual(package["engines"], {"node": ">=18"})
        self.assertTrue((out / "jag" / "bin" / "jag.js").is_file())
        self.assertTrue((out / "jag" / "LICENSE").is_file())

    def test_each_platform_package_has_one_os_and_one_cpu(self):
        for platform in release.PLATFORMS:
            out = self.build(platform["target"])
            folder = out / f"jag-{platform['os']}-{platform['cpu']}"
            package = json.loads((folder / "package.json").read_text())
            self.assertEqual(package["name"], f"@airf/jag-{platform['os']}-{platform['cpu']}")
            self.assertEqual(package["os"], [platform["os"]])
            self.assertEqual(package["cpu"], [platform["cpu"]])
            self.assertEqual(package["version"], VERSION)
            for forbidden in ("libc", "bin", "scripts"):
                self.assertNotIn(forbidden, package)
            self.assertEqual(stat.S_IMODE((folder / "bin" / "jag").stat().st_mode), 0o755)
            self.assertEqual(package["files"], ["bin/jag", "THIRD-PARTY-LICENSES"])
            self.assertEqual((folder / "THIRD-PARTY-LICENSES").read_text(), "THIRD-PARTY LICENSES\n")
            readme = (folder / "README.md").read_text()
            self.assertIn(package["name"], readme)
            self.assertIn(platform["label"], readme)
            self.assertNotIn("{package}", readme)

    def test_each_package_keeps_the_license_file_of_the_root_byte_for_byte(self):
        out = self.tmp / "out"
        binary = write(self.tmp / "bin" / "jag", b"\x7fELF", 0o755)
        notices = write(self.tmp / "THIRD-PARTY-LICENSES", "THIRD-PARTY LICENSES\n")
        for platform in release.PLATFORMS:
            code, _, err = run_main(
                ["npm-platform", "--target", platform["target"], "--version", VERSION, "--binary", str(binary),
                 "--out", str(out), "--readme", str(ROOT / "npm" / "platform" / "README.md"),
                 "--license", str(ROOT / "LICENSE"), "--notices", str(notices)]
            )
            self.assertEqual(code, 0, err)
            folder = out / f"jag-{platform['os']}-{platform['cpu']}"
            self.assertEqual((folder / "LICENSE").read_bytes(), (ROOT / "LICENSE").read_bytes())
        code, _, err = run_main(
            ["npm-main", "--version", VERSION, "--out", str(out), "--source", str(ROOT / "npm" / "jag"),
             "--readme", str(ROOT / "README.md"), "--license", str(ROOT / "LICENSE")]
        )
        self.assertEqual(code, 0, err)
        self.assertEqual((out / "jag" / "LICENSE").read_bytes(), (ROOT / "LICENSE").read_bytes())

    def test_a_pre_release_version_is_refused(self):
        code, _, err = run_main(["npm-main", "--version", "1.0.0-rc.1", "--out", str(self.tmp / "o")])
        self.assertEqual(code, 1)
        self.assertIn("MAJOR.MINOR.PATCH", err)

    def tarball(self, name, package, files):
        members = [("package/package.json", json.dumps(package).encode(), 0o644)]
        members += [(f"package/{path}", data, mode) for path, (data, mode) in files.items()]
        return make_tgz(self.tmp / name, members)

    def good_platform(self):
        package = release.npm_platform_package(release.PLATFORMS[0], VERSION)
        files = {
            "bin/jag": (b"\x7fELF", 0o755),
            "README.md": (b"r", 0o644),
            "LICENSE": (b"l", 0o644),
            "THIRD-PARTY-LICENSES": (b"THIRD-PARTY LICENSES\n", 0o644),
        }
        return package, files

    def good_main(self):
        package = release.npm_main_package(json.loads((ROOT / "npm/jag/package.json").read_text()), VERSION)
        files = {"bin/jag.js": (b"#!/usr/bin/env node\n", 0o755), "README.md": (b"r", 0o644), "LICENSE": (b"l", 0o644)}
        return package, files

    def check(self, tarball):
        return run_main(["check-npm-tarball", "--version", VERSION, str(tarball)])

    def test_a_good_platform_tarball_and_a_good_main_tarball_pass(self):
        package, files = self.good_platform()
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 0)
        package, files = self.good_main()
        self.assertEqual(self.check(self.tarball("m.tgz", package, files))[0], 0)

    def test_an_install_script_fails(self):
        for script in release.INSTALL_SCRIPTS:
            package, files = self.good_platform()
            package["scripts"] = {script: "node x.js"}
            code, _, err = self.check(self.tarball(f"{script}.tgz", package, files))
            self.assertEqual(code, 1, script)
            self.assertIn(script, err)

    def test_a_binary_with_no_execute_bit_fails(self):
        package, files = self.good_platform()
        files["bin/jag"] = (b"\x7fELF", 0o644)
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    def test_an_extra_file_fails(self):
        package, files = self.good_platform()
        files["docs/x.md"] = (b"x", 0o644)
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    def test_a_wrong_repository_url_fails(self):
        package, files = self.good_platform()
        package["repository"] = {"type": "git", "url": "git+https://example.com/x.git"}
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    def test_a_platform_tarball_with_no_notice_file_fails(self):
        package, files = self.good_platform()
        del files["THIRD-PARTY-LICENSES"]
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    def test_a_main_tarball_with_a_notice_file_fails(self):
        package, files = self.good_main()
        files["THIRD-PARTY-LICENSES"] = (b"x", 0o644)
        self.assertEqual(self.check(self.tarball("m.tgz", package, files))[0], 1)

    def test_a_wrong_license_repository_or_description_fails(self):
        for good in (self.good_platform, self.good_main):
            for change in (
                {"license": "Apache-2.0"},
                {"license": None},
                {"repository": {"type": "git", "url": "https://github.com/ai-ready-future/jag"}},
                {"repository": None},
                {"description": "Something else"},
                {"description": None},
            ):
                package, files = good()
                package.update(change)
                code, _, err = self.check(self.tarball("x.tgz", package, files))
                self.assertEqual(code, 1, (good.__name__, change))
                self.assertTrue(err.startswith("release:"), err)

    def test_the_description_of_the_main_package_is_the_description_of_the_crate(self):
        package = json.loads((ROOT / "npm" / "jag" / "package.json").read_text())
        self.assertEqual(package["description"], release.cargo_metadata(ROOT / "Cargo.toml")["description"])

    def test_a_libc_value_or_two_cpu_values_fail(self):
        for change in ({"libc": ["musl"]}, {"cpu": ["x64", "arm64"]}, {"bin": {"jag": "bin/jag"}}):
            package, files = self.good_platform()
            package.update(change)
            self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1, change)

    def test_a_main_package_with_a_wrong_dependency_version_fails(self):
        package, files = self.good_main()
        package["optionalDependencies"]["@airf/jag-linux-x64"] = "9.9.9"
        self.assertEqual(self.check(self.tarball("m.tgz", package, files))[0], 1)

    def test_a_wrong_version_fails(self):
        package, files = self.good_platform()
        package["version"] = "9.9.9"
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    def test_an_unknown_package_name_fails(self):
        package, files = self.good_platform()
        package["name"] = "@airf/other"
        self.assertEqual(self.check(self.tarball("p.tgz", package, files))[0], 1)

    @unittest.skipUnless(shutil.which("npm") and shutil.which("node"), "needs npm and node")
    def test_npm_pack_keeps_the_execute_bit_and_the_checks_pass(self):
        out = self.build()
        pack = self.tmp / "pack"
        pack.mkdir()
        for folder in (out / "jag-linux-x64", out / "jag"):
            result = subprocess.run(
                ["npm", "pack", str(folder), "--pack-destination", str(pack)],
                capture_output=True, text=True, env={**os.environ, "npm_config_cache": str(self.tmp / "cache")},
            )
            self.assertEqual(result.returncode, 0, result.stderr)
        platform_file = pack / f"airf-jag-linux-x64-{VERSION}.tgz"
        main_file = pack / f"airf-jag-{VERSION}.tgz"
        self.assertEqual(self.check(platform_file)[0], 0)
        self.assertEqual(self.check(main_file)[0], 0)

    @unittest.skipUnless(shutil.which("npm") and shutil.which("node"), "needs npm and node")
    def test_local_install_runs_the_launcher_and_a_fake_binary(self):
        """The launcher and a fake jag run as npm installs them (the job build does this with the real binary)."""
        platform = next(
            p for p in release.PLATFORMS if f"{p['os']} {p['cpu']}" == self.node_platform()
        ) if self.node_platform() else None
        if platform is None:
            self.skipTest("this machine is not one of the four platforms")
        out = self.tmp / "out"
        binary = write(
            self.tmp / "fake-jag",
            f'#!/bin/sh\ncase "$1" in\n  --version) echo "jag {VERSION}" ;;\n  *) echo "bad option" >&2; exit 2 ;;\nesac\n',
            0o755,
        )
        license_file = write(self.tmp / "LICENSE", "license\n")
        notices = write(self.tmp / "THIRD-PARTY-LICENSES", "THIRD-PARTY LICENSES\n")
        run_main(["npm-main", "--version", VERSION, "--out", str(out), "--source", str(ROOT / "npm/jag"),
                  "--readme", str(ROOT / "README.md"), "--license", str(license_file)])
        run_main(["npm-platform", "--target", platform["target"], "--version", VERSION, "--binary", str(binary),
                  "--out", str(out), "--readme", str(ROOT / "npm/platform/README.md"), "--license", str(license_file),
                  "--notices", str(notices)])
        pack = self.tmp / "pack"
        pack.mkdir()
        for folder in (out / f"jag-{platform['os']}-{platform['cpu']}", out / "jag"):
            subprocess.run(
                ["npm", "pack", str(folder), "--pack-destination", str(pack)], check=True, capture_output=True,
                env={**os.environ, "npm_config_cache": str(self.tmp / "cache")},
            )
        code, _, err = run_main(
            [
                "local-install", "--main", str(pack / f"airf-jag-{VERSION}.tgz"),
                "--platform", str(pack / f"airf-jag-{platform['os']}-{platform['cpu']}-{VERSION}.tgz"),
                "--version", VERSION,
            ]
        )
        self.assertEqual(code, 0, err)

    @staticmethod
    def node_platform():
        result = subprocess.run(
            ["node", "-p", "process.platform + ' ' + process.arch"], capture_output=True, text=True
        )
        return result.stdout.strip() if result.returncode == 0 else None


class VersionCheckTests(unittest.TestCase):
    def test_npm_versions(self):
        for good in ["11.5.1", "11.15.0", "12.0.0", "11.5.1\n"]:
            self.assertEqual(run_main(["check-npm-version", good])[0], 0, good)
        for bad in ["11.5.0", "10.9.2", "9.0.0"]:
            self.assertEqual(run_main(["check-npm-version", bad])[0], 1, bad)

    def test_npm_install_log(self):
        good = (
            "npm http fetch GET 200 https://registry.npmjs.org/@airf%2fjag 200ms\n"
            "npm http fetch GET 200 https://registry.npmjs.org/@airf/jag/-/jag-1.2.3.tgz 100ms\n"
            "added 2 packages\n"
        )
        self.assertEqual(release.check_install_log(good), 2)
        with self.assertRaises(release.ReleaseError):
            release.check_install_log(good + "npm http fetch GET 200 https://github.com/x/y 1ms\n")
        with self.assertRaises(release.ReleaseError):
            release.check_install_log("added 2 packages\n")
        with self.assertRaises(release.ReleaseError):
            release.check_install_log("npm http fetch GET 200 https://registry.npmjs.org/a 1ms\n")


class CrateListTests(unittest.TestCase):
    GOOD = ["Cargo.lock", "Cargo.toml", "Cargo.toml.orig", "LICENSE", "README.md", "src/lib.rs", "src/cli/stop.rs"]

    def test_a_good_list_passes(self):
        release.check_crate_list(self.GOOD)
        release.check_crate_list([*self.GOOD, ".cargo_vcs_info.json"])

    def test_a_path_outside_the_list_fails_with_the_line_number_and_not_the_path(self):
        code = release.ReleaseError
        with self.assertRaises(code) as caught:
            release.check_crate_list([*self.GOOD[:3], "docs/x.md", *self.GOOD[3:]])
        self.assertIn("line 4", str(caught.exception))
        self.assertNotIn("docs", str(caught.exception))

    def test_a_missing_required_file_fails(self):
        with self.assertRaises(release.ReleaseError):
            release.check_crate_list(["Cargo.toml", "src/lib.rs"])

    def test_the_command_reads_a_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = write(Path(tmp) / "list.txt", "\n".join(self.GOOD) + "\n")
            self.assertEqual(run_main(["check-crate-list", str(path)])[0], 0)
            bad = write(Path(tmp) / "bad.txt", "\n".join([*self.GOOD, "CLAUDE.md"]) + "\n")
            self.assertEqual(run_main(["check-crate-list", str(bad)])[0], 1)

    def test_the_include_list_of_cargo_toml(self):
        import tomllib

        with open(ROOT / "Cargo.toml", "rb") as handle:
            package = tomllib.load(handle)["package"]
        self.assertEqual(
            package["include"], ["/src/**", "/Cargo.toml", "/Cargo.lock", "/README.md", "/LICENSE"]
        )
        self.assertEqual(package["readme"], "README.md")
        self.assertTrue(package["keywords"])
        self.assertTrue(package["categories"])


class CrateMetadataTests(TempCase):
    def crate(self, name="jag-cli-1.2.3.crate", manifest=None, folder="jag-cli-1.2.3"):
        text = (ROOT / "Cargo.toml").read_text(encoding="utf-8") if manifest is None else manifest
        return make_tgz(self.tmp / name, [(f"{folder}/Cargo.toml", text.encode(), 0o644)])

    def with_value(self, key, value):
        text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        lines = [f'{key} = "{value}"' if line.startswith(f"{key} = ") else line for line in text.splitlines()]
        self.assertNotEqual("\n".join(lines), text.rstrip("\n"), key)
        return "\n".join(lines) + "\n"

    def test_the_crate_of_the_repository_passes(self):
        code, out, err = run_main(["check-crate-metadata", str(self.crate())])
        self.assertEqual((code, err), (0, ""))
        self.assertIn("passes", out)

    def test_the_metadata_of_the_repository_is_the_metadata_of_the_spec(self):
        found = release.cargo_metadata(ROOT / "Cargo.toml")
        self.assertEqual(found["license"], "MIT")
        self.assertEqual(found["repository"], "https://github.com/ai-ready-future/jag")
        self.assertTrue(found["description"])

    def test_a_wrong_license_fails(self):
        code, _, err = run_main(["check-crate-metadata", str(self.crate(manifest=self.with_value("license", "Apache-2.0")))])
        self.assertEqual(code, 1)
        self.assertIn("license", err)

    def test_a_wrong_repository_fails(self):
        manifest = self.with_value("repository", "https://example.com/x")
        code, _, err = run_main(["check-crate-metadata", str(self.crate(manifest=manifest))])
        self.assertEqual(code, 1)
        self.assertIn("repository", err)

    def test_a_wrong_description_fails(self):
        manifest = self.with_value("description", "Something else")
        code, _, err = run_main(["check-crate-metadata", str(self.crate(manifest=manifest))])
        self.assertEqual(code, 1)
        self.assertIn("description", err)

    def test_a_missing_value_fails(self):
        manifest = '[package]\nname = "jag-cli"\nversion = "1.2.3"\n'
        self.assertEqual(run_main(["check-crate-metadata", str(self.crate(manifest=manifest))])[0], 1)

    def test_a_crate_with_no_manifest_in_its_folder_fails(self):
        path = make_tgz(self.tmp / "x.crate", [("jag-cli-1.2.3/src/Cargo.toml", b"x", 0o644)])
        self.assertEqual(run_main(["check-crate-metadata", str(path)])[0], 1)


class NoticesTests(TempCase):
    def check(self, data):
        path = write(self.tmp / "THIRD-PARTY-LICENSES", data)
        return run_main(["check-notices", str(path)])

    def test_a_good_file_passes(self):
        code, out, err = self.check("THIRD-PARTY LICENSES\n\nThe jag binary holds code.\n")
        self.assertEqual((code, err), (0, ""))
        self.assertIn("passes", out)

    def test_an_empty_file_fails(self):
        self.assertEqual(self.check("")[0], 1)
        self.assertEqual(self.check("\n \n")[0], 1)

    def test_a_wrong_first_line_fails(self):
        self.assertEqual(self.check("THIRD-PARTY LICENCES\nx\n")[0], 1)
        self.assertEqual(self.check("\nTHIRD-PARTY LICENSES\n")[0], 1)

    def test_a_file_that_is_not_utf8_fails(self):
        self.assertEqual(self.check(b"THIRD-PARTY LICENSES\n\xff\xfe\n")[0], 1)


class RepositoryFilesTests(unittest.TestCase):
    def test_the_license_file_is_mit_and_the_notice_files_exist(self):
        self.assertTrue((ROOT / "LICENSE").read_text(encoding="utf-8").startswith("MIT License"))
        self.assertTrue((ROOT / "scripts" / "release" / "about.toml").is_file())
        self.assertTrue((ROOT / "scripts" / "release" / "notices.hbs").is_file())

    def test_the_notice_file_is_not_stored_in_the_repository(self):
        self.assertFalse((ROOT / "THIRD-PARTY-LICENSES").exists())

    def test_the_accepted_licenses_of_about_toml_agree_with_the_spec(self):
        with open(ROOT / "scripts" / "release" / "about.toml", "rb") as handle:
            config = tomllib.load(handle)
        self.assertEqual(
            config["accepted"],
            ["MIT", "Apache-2.0", "ISC", "BSD-3-Clause", "Unicode-3.0", "CDLA-Permissive-2.0"],
        )
        self.assertEqual(sorted(config["targets"]), sorted(p["target"] for p in release.PLATFORMS))
        self.assertTrue(config["ignore-dev-dependencies"] and config["ignore-build-dependencies"])

    def test_the_template_starts_with_the_line_that_check_notices_wants(self):
        first = (ROOT / "scripts" / "release" / "notices.hbs").read_text(encoding="utf-8").split("\n", 1)[0]
        self.assertEqual(first, release.NOTICES_FIRST_LINE)


class TextsTests(TempCase):
    def run_texts(self, *files):
        text, index = self.tmp / "text", self.tmp / "index"
        code, _, err = run_main(["texts", "--text", str(text), "--index", str(index), *map(str, files)])
        self.assertEqual(code, 0, err)
        return text.read_text().splitlines(), index.read_text().splitlines()

    def test_a_text_member_a_binary_member_and_a_member_name(self):
        archive = make_tgz(
            self.tmp / "jag-1.2.3-x.tar.gz",
            [
                ("folder", None, 0o755),
                ("folder/SECRET-NAME.txt", b"first line\nsecond line\n", 0o644),
                ("folder/jag", b"\x00\x01binary-string-here\x00\x02ab\x00longer string", 0o755),
            ],
        )
        text, index = self.run_texts(archive)
        self.assertEqual(len(text), len(index))
        self.assertIn("folder", text)
        self.assertIn("folder/SECRET-NAME.txt", text)
        self.assertIn("first line", text)
        self.assertIn("second line", text)
        self.assertIn("binary-string-here", text)
        self.assertIn("longer string", text)
        self.assertNotIn("ab", text, "a run of fewer than 4 bytes gives no line")
        labels = dict(zip(text, index))
        self.assertEqual(labels["folder"], "jag-1.2.3-x.tar.gz member 1 name")
        self.assertEqual(labels["folder/SECRET-NAME.txt"], "jag-1.2.3-x.tar.gz member 2 name")
        self.assertEqual(labels["first line"], "jag-1.2.3-x.tar.gz member 2 line 1")
        self.assertEqual(labels["second line"], "jag-1.2.3-x.tar.gz member 2 line 2")
        self.assertEqual(labels["binary-string-here"], "jag-1.2.3-x.tar.gz member 3 byte 2")
        self.assertTrue(labels["longer string"].startswith("jag-1.2.3-x.tar.gz member 3 byte "))

    def test_no_index_line_holds_a_member_path_or_a_value(self):
        archive = make_tgz(
            self.tmp / "a.tar.gz",
            [("secret/dir/file.txt", b"VALUE-SHOULD-NOT-LEAK\n", 0o644)],
        )
        _, index = self.run_texts(archive)
        for line in index:
            self.assertNotIn("secret", line)
            self.assertNotIn("file.txt", line)
            self.assertNotIn("VALUE", line)

    def test_a_sha256_file_is_text(self):
        checksum = write(self.tmp / "a.tar.gz.sha256", f"{'a' * 64}  a.tar.gz\n")
        text, index = self.run_texts(checksum)
        self.assertEqual(text, [f"{'a' * 64}  a.tar.gz"])
        self.assertEqual(index, ["a.tar.gz.sha256 line 1"])

    def test_a_bad_utf8_byte_becomes_a_replacement_character(self):
        archive = make_tgz(self.tmp / "a.tar.gz", [("f.txt", b"ok \xff\xfe end\n", 0o644)])
        text, _ = self.run_texts(archive)
        self.assertIn("ok �� end", text)

    def test_a_name_with_a_newline_keeps_one_index_line_for_each_text_line(self):
        archive = make_tgz(self.tmp / "a.tar.gz", [("one\ntwo", b"x\n", 0o644)])
        text, index = self.run_texts(archive)
        self.assertEqual(len(text), len(index))
        self.assertEqual(text[:2], ["one", "two"])

    def test_the_line_counts_agree_for_a_file_with_no_final_newline(self):
        archive = make_tgz(self.tmp / "a.tar.gz", [("f.txt", b"a\nb", 0o644)])
        text, index = self.run_texts(archive)
        self.assertEqual(len(text), len(index))
        self.assertIn("b", text)

    def test_a_crate_file_is_read_as_a_tar_gz(self):
        crate = make_tgz(self.tmp / "jag-cli-1.2.3.crate", [("jag-cli-1.2.3/Cargo.toml", b"[package]\n", 0o644)])
        text, index = self.run_texts(crate)
        self.assertIn("[package]", text)
        self.assertIn("jag-cli-1.2.3.crate member 1 line 1", index)


class ReleaseFilesTests(TempCase):
    def fill(self, names=None):
        for platform in release.PLATFORMS:
            name = release.archive_name(VERSION, platform["target"])
            data = name.encode()
            write(self.tmp / name, data)
            write(self.tmp / f"{name}.sha256", f"{hashlib.sha256(data).hexdigest()}  {name}\n")

    def test_a_good_release_passes(self):
        self.fill()
        self.assertEqual(run_main(["check-release", "--dir", str(self.tmp), "--version", VERSION])[0], 0)

    def test_a_missing_file_fails(self):
        self.fill()
        next(self.tmp.glob("*.sha256")).unlink()
        self.assertEqual(run_main(["check-release", "--dir", str(self.tmp), "--version", VERSION])[0], 1)

    def test_an_extra_file_fails(self):
        self.fill()
        write(self.tmp / "extra.txt", "x")
        self.assertEqual(run_main(["check-release", "--dir", str(self.tmp), "--version", VERSION])[0], 1)

    def test_a_checksum_that_does_not_agree_fails(self):
        self.fill()
        name = release.archive_name(VERSION, release.PLATFORMS[0]["target"])
        write(self.tmp / name, b"changed")
        self.assertEqual(run_main(["check-release", "--dir", str(self.tmp), "--version", VERSION])[0], 1)

    def test_a_checksum_with_one_space_fails(self):
        self.fill()
        name = release.archive_name(VERSION, release.PLATFORMS[0]["target"])
        digest = hashlib.sha256((self.tmp / name).read_bytes()).hexdigest()
        write(self.tmp / f"{name}.sha256", f"{digest} {name}\n")
        self.assertEqual(run_main(["check-release", "--dir", str(self.tmp), "--version", VERSION])[0], 1)


class ExistsTests(unittest.TestCase):
    class Response:
        def __init__(self, status):
            self.status = status

        def __enter__(self):
            return self

        def __exit__(self, *args):
            return False

    def opener(self, status=None, error=None, seen=None):
        def open_url(request, timeout):
            if seen is not None:
                seen.append(request)
            if error is not None:
                raise error
            return self.Response(status)

        return open_url

    def test_the_urls(self):
        self.assertEqual(
            release.registry_url("crates", "jag-cli", VERSION), "https://crates.io/api/v1/crates/jag-cli/1.2.3"
        )
        self.assertEqual(
            release.registry_url("npm", "@airf/jag-linux-x64", VERSION),
            "https://registry.npmjs.org/@airf%2fjag-linux-x64/1.2.3",
        )

    def test_found_and_not_found(self):
        seen = []
        self.assertTrue(release.version_exists("crates", "jag-cli", VERSION, self.opener(200, seen=seen)))
        self.assertIn("User-Agent", {k.title() if k.lower() == "user-agent" else k for k in seen[0].headers})
        not_found = urllib.error.HTTPError("u", 404, "Not Found", {}, None)
        self.addCleanup(not_found.close)
        self.assertFalse(release.version_exists("npm", "@airf/jag", VERSION, self.opener(error=not_found)))

    def test_another_answer_is_an_error_and_never_a_skip(self):
        server_error = urllib.error.HTTPError("u", 500, "Error", {}, None)
        self.addCleanup(server_error.close)
        with self.assertRaises(release.ReleaseError):
            release.version_exists("npm", "@airf/jag", VERSION, self.opener(error=server_error))
        with self.assertRaises(release.ReleaseError):
            release.version_exists("npm", "@airf/jag", VERSION, self.opener(error=urllib.error.URLError("down")))


def job_blocks(text):
    """The text of each job of a workflow file. A job starts at two spaces, a name, and a colon, under jobs:."""
    lines = text.splitlines()
    start = lines.index("jobs:")
    blocks, name = {}, None
    for line in lines[start + 1 :]:
        match = re.match(r"^  ([a-z][a-z0-9-]*):\s*$", line)
        if match:
            name = match.group(1)
            blocks[name] = []
        elif name is not None:
            blocks[name].append(line)
    return {key: "\n".join(value) for key, value in blocks.items()}


class WorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text(encoding="utf-8")
        cls.jobs = job_blocks(cls.text)

    def test_the_jobs_of_the_spec(self):
        self.assertEqual(
            list(self.jobs),
            [
                "prepare", "npm-main", "notices", "build", "crate", "deps", "scan",
                "github-draft", "crates-io", "npm", "verify", "github-publish",
            ],
        )

    def test_the_notices_job_pins_cargo_about_and_makes_the_file_for_the_build_jobs(self):
        notices = self.jobs["notices"]
        self.assertIn("cargo-about-0.9.2-x86_64-unknown-linux-musl.tar.gz", notices)
        self.assertIn("9099a59e820c38a68b9d65f300662a567d56562f9a10f6aa4c7e86c17c2566af", notices)
        self.assertIn("sha256sum -c", notices)
        self.assertIn("generate --locked --fail", notices)
        self.assertIn("scripts/release/about.toml", notices)
        self.assertIn("scripts/release/notices.hbs", notices)
        self.assertIn("check-notices", notices)
        self.assertIn("name: notices", notices)
        self.assertNotIn("secrets.", notices)
        self.assertIn("needs: [prepare, npm-main, notices]", self.jobs["build"])
        build = self.jobs["build"]
        self.assertEqual(build.count("--notices "), 2)
        self.assertIn("release.py archive", build)
        self.assertIn("release.py npm-platform", build)

    def test_the_crate_job_checks_the_metadata_of_the_packed_crate(self):
        self.assertIn("check-crate-metadata", self.jobs["crate"])

    def test_the_only_secrets_are_private_patterns_and_the_github_token(self):
        names = set(re.findall(r"secrets\.([A-Za-z0-9_]+)", self.text))
        self.assertEqual(names, {"PRIVATE_PATTERNS", "GITHUB_TOKEN"})
        self.assertEqual(
            [name for name, block in self.jobs.items() if "secrets.PRIVATE_PATTERNS" in block], ["scan"]
        )

    def test_the_private_patterns_reach_only_the_scan_step_of_a_tag_or_a_manual_run_on_main(self):
        # FEATURE-18 spec decision 28 and the security issue #161: a dry run of a pull request gets no secret.
        condition = "github.event_name == 'push' || (github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main')"
        self.assertEqual(self.text.count("secrets.PRIVATE_PATTERNS"), 1)
        steps = re.split(r"\n      - ", self.jobs["scan"])
        holders = [step for step in steps if "secrets.PRIVATE_PATTERNS" in step]
        self.assertEqual(len(holders), 1)
        step = holders[0]
        self.assertTrue(step.startswith("name: Scan the packages"), step[:60])
        self.assertIn(f"\n        if: {condition}\n", step)
        # The condition names no pull request event, so a dry run of a pull request skips the step.
        self.assertNotIn("pull_request", condition)

    def test_only_the_two_publish_jobs_have_id_token_write(self):
        self.assertNotIn("id-token", self.text.split("jobs:")[0])
        self.assertEqual(
            sorted(name for name, block in self.jobs.items() if "id-token: write" in block), ["crates-io", "npm"]
        )

    def test_no_job_has_contents_write_and_id_token_write(self):
        for name, block in self.jobs.items():
            self.assertFalse("contents: write" in block and "id-token: write" in block, name)
        self.assertEqual(
            sorted(name for name, block in self.jobs.items() if "contents: write" in block),
            ["github-draft", "github-publish"],
        )

    def test_the_default_permission_is_contents_read(self):
        head = self.text.split("jobs:")[0]
        self.assertRegex(head, r"\npermissions:\n  contents: read\n")

    def test_no_step_sets_a_token_for_npm(self):
        code = "\n".join(line for line in self.text.splitlines() if not line.lstrip().startswith("#"))
        self.assertFalse("NODE_AUTH_TOKEN" in code, "a step sets NODE_AUTH_TOKEN")
        self.assertFalse("NPM_TOKEN" in code, "a step sets NPM_TOKEN")

    def test_the_publish_jobs_run_only_for_a_tag(self):
        for name in ["github-draft", "crates-io", "npm", "verify", "github-publish"]:
            self.assertIn("needs.prepare.outputs.publish == 'true'", self.jobs[name], name)
        for name in ["prepare", "npm-main", "build", "crate", "deps", "scan"]:
            self.assertNotIn("environment:", self.jobs[name], name)
        self.assertIn('if [ "$EVENT_NAME" = push ]', self.jobs["prepare"])

    def test_the_jobs_that_publish_use_the_environment_release(self):
        for name in ["github-draft", "crates-io", "npm", "github-publish"]:
            self.assertIn("environment: release", self.jobs[name], name)

    def test_the_cargo_token_is_in_one_step_only(self):
        self.assertEqual(self.text.count("CARGO_REGISTRY_TOKEN"), 1)

    def test_each_action_has_a_full_commit_sha_and_each_checkout_keeps_no_credentials(self):
        for match in re.finditer(r"uses: (\S+)@(\S+)", self.text):
            self.assertRegex(match.group(2), r"^[0-9a-f]{40}$", match.group(0))
        self.assertEqual(
            self.text.count("actions/checkout@"), self.text.count("persist-credentials: false")
        )

    def test_the_workflow_uses_no_cache_and_no_latest_runner(self):
        self.assertFalse("actions/cache" in self.text, "a job uses a cache")
        self.assertIsNone(re.search(r"(ubuntu|macos|windows)-latest", self.text))

    def test_the_triggers(self):
        head = self.text.split("jobs:")[0]
        for needle in [
            "tags: ['v*']", "workflow_dispatch:", "pull_request:", "- npm/**", "- scripts/release/**",
            "- deny.toml", "- scripts/deps/**",
        ]:
            self.assertIn(needle, head)

    def test_the_npm_job_publishes_the_platform_packages_before_the_main_package(self):
        order = re.search(r"for name in ([a-z0-9 -]+); do", self.jobs["npm"]).group(1).split()
        self.assertEqual(order[-1], "jag")
        self.assertEqual(sorted(order[:-1]), sorted(f"jag-{p['os']}-{p['cpu']}" for p in release.PLATFORMS))

    def test_the_draft_waits_for_the_dependency_checks(self):
        self.assertRegex(self.jobs["github-draft"], r"needs: \[[^\]]*\bdeps\b[^\]]*\]")
        self.assertRegex(self.jobs["deps"], r"needs: prepare\n")
        self.assertIn("run: scripts/deps/check.sh", self.jobs["deps"])
        self.assertNotIn("secrets.", self.jobs["deps"])
        self.assertNotIn("permissions:", self.jobs["deps"])

    def test_ci_runs_the_tests_of_the_tools(self):
        ci = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("python3 -I -m unittest discover -s scripts/release -p 'test_*.py'", ci)
        self.assertIn("node --test npm/jag/test/launcher.test.js", ci)


class DepsWorkflowTests(unittest.TestCase):
    """CRITERION-106: the dependency checks run on each pull request, each push to main, and each release."""

    @classmethod
    def setUpClass(cls):
        cls.text = DEPS_WORKFLOW.read_text(encoding="utf-8")
        cls.jobs = job_blocks(cls.text)
        cls.script = (ROOT / "scripts" / "deps" / "check.sh").read_text(encoding="utf-8")
        cls.deny = tomllib.loads((ROOT / "deny.toml").read_text(encoding="utf-8"))

    def test_the_triggers_have_no_schedule(self):
        head = self.text.split("jobs:")[0]
        for needle in ["pull_request:", "push:\n    branches: [main]", "workflow_dispatch:"]:
            self.assertIn(needle, head)
        self.assertNotIn("schedule", head.replace("No schedule event", ""))
        self.assertNotIn("cron", self.text)

    def test_the_workflow_has_one_job_that_only_reads(self):
        self.assertEqual(list(self.jobs), ["deps"])
        self.assertRegex(self.text.split("jobs:")[0], r"\npermissions:\n  contents: read\n")
        self.assertNotIn("write", self.text.replace("no write", ""))
        self.assertNotIn("secrets.", self.text)
        self.assertFalse("actions/cache" in self.text, "the job uses a cache")
        self.assertIn("run: scripts/deps/check.sh", self.jobs["deps"])

    def test_each_action_has_a_full_commit_sha_and_the_checkout_keeps_no_credentials(self):
        for match in re.finditer(r"uses: (\S+)@(\S+)", self.text):
            self.assertRegex(match.group(2), r"^[0-9a-f]{40}$", match.group(0))
        self.assertEqual(self.text.count("actions/checkout@"), self.text.count("persist-credentials: false"))

    def test_the_script_can_run_and_pins_cargo_deny_by_sha256(self):
        path = ROOT / "scripts" / "deps" / "check.sh"
        self.assertTrue(os.access(path, os.X_OK), "check.sh is not executable")
        self.assertRegex(self.script, r"\nCARGO_DENY_SHA256=[0-9a-f]{64}\n")
        self.assertIn("sha256sum -c", self.script)
        self.assertIn("--locked check advisories licenses bans sources", self.script)
        self.assertIn("npm audit\n", self.script)
        self.assertNotIn("--audit-level", self.script)
        self.assertIn("python3 -I scripts/release/release.py check-npm-deps npm/jag/package.json", self.script)

    def test_the_policy_checks_the_four_release_targets_and_no_dev_dependency(self):
        self.assertEqual(
            sorted(self.deny["graph"]["targets"]), sorted(p["target"] for p in release.PLATFORMS)
        )
        self.assertIs(self.deny["graph"]["exclude-dev"], True)

    def test_each_kind_of_advisory_fails_the_check(self):
        advisories = self.deny["advisories"]
        self.assertEqual(advisories["yanked"], "deny")
        self.assertEqual(advisories["unmaintained"], "all")
        self.assertEqual(advisories["unsound"], "all")
        self.assertNotIn("severity-threshold", advisories)

    def test_each_ignore_entry_names_an_advisory_and_an_issue(self):
        for entry in self.deny["advisories"]["ignore"]:
            self.assertRegex(entry["id"], r"^RUSTSEC-\d{4}-\d{4}$")
            self.assertRegex(entry["reason"], r"^#\d+: (medium|low), ", entry["id"])

    def test_the_licenses_are_the_six_of_the_spec_and_none_is_unused(self):
        self.assertEqual(
            sorted(self.deny["licenses"]["allow"]),
            ["Apache-2.0", "BSD-3-Clause", "CDLA-Permissive-2.0", "ISC", "MIT", "Unicode-3.0"],
        )
        self.assertEqual(self.deny["licenses"]["unused-allowed-license"], "deny")
        self.assertEqual(self.deny["sources"], {"unknown-registry": "deny", "unknown-git": "deny"})


class CheckNpmDepsTests(TempCase):
    """CRITERION-106: the npm packages have no third-party dependency, so npm audit has nothing to find."""

    PLATFORM_NAMES = [release.npm_platform_name(p) for p in release.PLATFORMS]

    def check(self, package):
        path = self.tmp / "package.json"
        path.write_text(json.dumps(package), encoding="utf-8")
        return run_main(["check-npm-deps", str(path)])

    def test_the_main_package_of_the_repository_passes(self):
        code, out, err = run_main(["check-npm-deps", str(ROOT / "npm" / "jag" / "package.json")])
        self.assertEqual(code, 0, err)
        self.assertIn("no third-party dependency", out)

    def test_a_package_with_no_dependency_field_passes(self):
        self.assertEqual(self.check({"name": "x"})[0], 0)

    def test_the_four_platform_packages_as_optional_dependencies_pass(self):
        self.assertEqual(self.check({"optionalDependencies": {n: "1.0.0" for n in self.PLATFORM_NAMES}})[0], 0)

    def test_a_dependency_fails(self):
        code, _, err = self.check({"dependencies": {"left-pad": "1.3.0"}})
        self.assertEqual(code, 1)
        self.assertIn("dependencies", err)

    def test_each_other_dependency_field_fails(self):
        for field in ["peerDependencies", "bundleDependencies", "bundledDependencies"]:
            code, _, err = self.check({field: []})
            self.assertEqual(code, 1, field)
            self.assertIn(field, err)

    def test_an_optional_dependency_that_is_not_a_platform_package_fails(self):
        code, _, err = self.check({"optionalDependencies": {"fsevents": "2.3.3"}})
        self.assertEqual(code, 1)
        self.assertIn("fsevents", err)
        self.assertEqual(self.check({"optionalDependencies": {"@airf/jag-linux-x64": "1.0.0", "@airf/other": "1.0.0"}})[0], 1)


class FirstPublishTests(TempCase):
    """Run first-publish.sh with fake cargo and npm commands. Nothing is published."""

    SCRIPT = ROOT / "scripts" / "release" / "first-publish.sh"

    def run_script(self, args, env_extra=None, stdin=""):
        bin_dir = self.tmp / "fakebin"
        calls = self.tmp / "calls.txt"
        calls.unlink(missing_ok=True)
        write(
            bin_dir / "cargo",
            f'#!/bin/sh\necho "cargo $* (token=${{CARGO_REGISTRY_TOKEN:+set}})" >> "{calls}"\n',
            0o755,
        )
        # FAKE_NPM_EXISTING: the package names that "npm view" finds. FAKE_NPM_FAIL_ON: a package that fails to publish.
        write(
            bin_dir / "npm",
            "#!/bin/sh\n"
            'if [ "$1" = view ]; then\n'
            '  case " $FAKE_NPM_EXISTING " in *" ${2%@*} "*) echo "${2##*@}" ;; esac\n'
            "  exit 0\n"
            "fi\n"
            "name=$(sed -n 's/.*\"name\": \"\\(.*\\)\".*/\\1/p' package.json)\n"
            f'echo "npm $* name=$name" >> "{calls}"\n'
            '[ "$name" != "$FAKE_NPM_FAIL_ON" ]\n',
            0o755,
        )
        # FAKE_CRATE_HTTP: the HTTP code that the crates.io check gets.
        write(bin_dir / "curl", '#!/bin/sh\necho "${FAKE_CRATE_HTTP:-404}"\n', 0o755)
        env = {"PATH": f"{bin_dir}:/usr/bin:/bin", "HOME": str(self.tmp / "home")}
        env.update(env_extra or {})
        result = subprocess.run(
            ["bash", str(self.SCRIPT), *args], capture_output=True, text=True, input=stdin, env=env, cwd=self.tmp
        )
        text = calls.read_text() if calls.exists() else ""
        return result, text.splitlines()

    def test_a_dry_run_asks_for_no_token_and_publishes_nothing(self):
        result, calls = self.run_script(["--dry-run"])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 6)
        self.assertEqual(calls[0], "cargo publish --dry-run (token=)")
        for line in calls[1:]:
            self.assertTrue(line.startswith("npm publish --dry-run --access public"), line)

    def test_the_order_and_the_token(self):
        result, calls = self.run_script([], {"CARGO_REGISTRY_TOKEN": "SENTINEL-TOKEN"}, stdin="yes\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls[0], "cargo publish (token=set)")
        self.assertEqual(len(calls), 6)
        self.assertNotIn("SENTINEL-TOKEN", result.stdout + result.stderr)
        self.assertNotIn("SENTINEL-TOKEN", "\n".join(calls))

    def test_a_second_run_ends_a_first_publish_that_stopped_part_way(self):
        env = {"CARGO_REGISTRY_TOKEN": "SENTINEL-TOKEN", "FAKE_NPM_FAIL_ON": "@airf/jag-linux-arm64"}
        first, calls = self.run_script([], env, stdin="yes\n")
        self.assertNotEqual(first.returncode, 0)
        self.assertEqual(
            [c.split(" (")[0].split(" name=")[-1] if c.startswith("npm") else "cargo" for c in calls],
            ["cargo", "@airf/jag-linux-x64", "@airf/jag-linux-arm64"],
        )
        # The crate and the first package are on the registries now. The second run has no token and needs none.
        second, calls = self.run_script(
            [], {"FAKE_CRATE_HTTP": "200", "FAKE_NPM_EXISTING": "@airf/jag-linux-x64"}, stdin="yes\n"
        )
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn("jag-cli 0.0.0 is on crates.io already. Skip.", second.stdout)
        self.assertIn("@airf/jag-linux-x64 0.0.0 is on npm already. Skip.", second.stdout)
        self.assertEqual(
            [c.split(" name=")[-1] for c in calls],
            ["@airf/jag-linux-arm64", "@airf/jag-darwin-arm64", "@airf/jag-darwin-x64", "@airf/jag"],
        )
        self.assertFalse(any(c.startswith("cargo") for c in calls))

    def test_a_run_with_each_version_on_the_registries_publishes_nothing(self):
        existing = " ".join(f"@airf/{n}" for n in ["jag-linux-x64", "jag-linux-arm64", "jag-darwin-arm64", "jag-darwin-x64", "jag"])
        result, calls = self.run_script([], {"FAKE_CRATE_HTTP": "200", "FAKE_NPM_EXISTING": existing}, stdin="yes\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, [])

    def test_an_answer_other_than_yes_stops_before_any_publish(self):
        result, calls = self.run_script([], {"CARGO_REGISTRY_TOKEN": "SENTINEL-TOKEN"}, stdin="no\n")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(calls, [])

    def test_an_unknown_option_stops(self):
        result, calls = self.run_script(["--publish-now"])
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])

    def test_the_script_has_no_key_file_no_email_and_no_trace(self):
        text = self.SCRIPT.read_text()
        self.assertNotIn("set -x", text)
        self.assertNotIn(".config/jag", text)
        self.assertNotIn(".cargo/credentials", text)
        self.assertIsNone(re.search(r"[\w.+-]+@[\w-]+\.[\w.]+", text))


if __name__ == "__main__":
    unittest.main()
