#!/usr/bin/env python3
"""Measure the time of a typical search with the real model service.

STORY-44, CRITERION-65: a typical search returns in under 5 seconds.

Python 3, standard library only. The script never reads, prints, or sets the
key: it copies the environment of the user and removes only the settings of
the model service, so jag finds the key as it always does.

Exit code: 0 when the target is met, 1 when it is not met, 2 when jag fails
or an argument is wrong.
"""
import argparse
import datetime
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

TAG = "go1.25.0"
COMMIT = "6e676ab2b809d46623acb5988248d95d1eb7939c"
REPOSITORY = "https://github.com/golang/go.git"
TARGET_SECONDS = 5.0
# The codes of a search that ended (IF-4).
ENDED = (0, 1, 7, 8)
# The settings that make jag use anything but its defaults and the direct
# route to the TypeSafe API.
REMOVED_SETTINGS = (
    "JAG_BASE_URL",
    "JAG_MODEL",
    "JAG_TIMEOUT",
    "JAG_RETRY_WAIT",
    "JAG_MAX_SCORED",
    "JAG_MAX_CHARS",
)
SEARCHES = (
    ("error", "where does the code retry after an error"),
    ("default", "where are the default settings read"),
    ("close", "what happens when a connection closes"),
    ("lock", "where can two goroutines wait on the same lock"),
    ("path", "how is a relative path made absolute"),
)


class Stop(Exception):
    """The script stops with exit code 2."""


def environment():
    """The environment of the user, with no setting of the model service."""
    env = dict(os.environ)
    for name in REMOVED_SETTINGS:
        env.pop(name, None)
    return env


def run_jag(jag, term, question, folder, env):
    """Run one search. Give the wall time and the exit code."""
    started = time.perf_counter()
    try:
        done = subprocess.run(
            [jag, "--ask", question, term, folder],
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise Stop(f"jag does not start: {error}") from error
    elapsed = time.perf_counter() - started
    if done.returncode not in ENDED:
        error = done.stderr.decode("utf-8", errors="replace").rstrip()
        raise Stop(f"jag exited with {done.returncode}.\n{error}")
    return elapsed, done.returncode


def clone_go(parent):
    """Clone Go at the tag, and check the commit. Give the folder."""
    folder = os.path.join(parent, "go")
    try:
        subprocess.run(
            ["git", "-c", "advice.detachedHead=false", "clone", "--depth", "1", "--branch", TAG, "--quiet", REPOSITORY, folder],
            check=True,
        )
        head = subprocess.run(
            ["git", "-C", folder, "rev-parse", "HEAD"],
            check=True,
            stdout=subprocess.PIPE,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise Stop(f"The clone of Go failed: {error}") from error
    if head != COMMIT:
        raise Stop(f"The tag {TAG} gives the commit {head}, and not the commit {COMMIT}.")
    return folder


def count_files(folder):
    """The count of the files of a folder, with no `.git`."""
    count = 0
    for _, names, files in os.walk(folder):
        if ".git" in names:
            names.remove(".git")
        count += len(files)
    return count


def jag_version(jag, env):
    try:
        done = subprocess.run(
            [jag, "--version"],
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise Stop(f"jag does not start: {error}") from error
    words = done.stdout.decode("utf-8", errors="replace").split()
    return words[-1] if words else "unknown"


def measure(jag, folder, runs, env):
    """Give one row for each search: the term, the times, and the codes."""
    rows = []
    for term, question in SEARCHES:
        run_jag(jag, term, question, folder, env)  # warm up the file cache
        times, codes = [], set()
        for _ in range(runs):
            elapsed, code = run_jag(jag, term, question, folder, env)
            times.append(elapsed)
            codes.add(code)
        rows.append((term, times, sorted(codes)))
    return rows


def report(rows, jag, folder, name, jag_env):
    """Print the table, the verdict, and the result line. Give the exit code."""
    print(f"{'search':<9} {'median':>8} {'slowest':>8}  exit codes")
    medians, slowest = [], 0.0
    for term, times, codes in rows:
        median = statistics.median(times)
        medians.append(median)
        slowest = max(slowest, max(times))
        listed = ", ".join(str(code) for code in codes)
        print(f"{term:<9} {median:>6.2f} s {max(times):>6.2f} s  {listed}")
    met = all(median < TARGET_SECONDS for median in medians)
    print(f"Target under 5 s: {'met' if met else 'not met'}.")
    system = f"{platform.system()} {platform.machine()}"
    files = f"{count_files(folder):,}"
    print(
        f"{datetime.date.today().isoformat()}: jag {jag_version(jag, jag_env)} on {system}, "
        f"{os.cpu_count()} CPUs, {name} ({files} files): "
        f"medians {min(medians):.2f} to {max(medians):.2f} s, slowest run {slowest:.2f} s."
    )
    return 0 if met else 1


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--jag", required=True, help="the jag program to measure")
    parser.add_argument("--repo", help="a folder to search (the default is a clone of Go 1.25.0)")
    parser.add_argument("--runs", type=int, default=5, help="the timed runs of each search")
    arguments = parser.parse_args(argv)
    try:
        if arguments.runs < 1:
            raise Stop("--runs must be 1 or more.")
        if arguments.repo is not None and not os.path.isdir(arguments.repo):
            raise Stop("--repo is not a folder.")
        env = environment()
        if arguments.repo is not None:
            return report(
                measure(arguments.jag, arguments.repo, arguments.runs, env),
                arguments.jag,
                arguments.repo,
                os.path.basename(os.path.abspath(arguments.repo)),
                env,
            )
        parent = tempfile.mkdtemp(prefix="jag-speed-")
        try:
            folder = clone_go(parent)
            return report(
                measure(arguments.jag, folder, arguments.runs, env),
                arguments.jag,
                folder,
                "Go 1.25.0",
                env,
            )
        finally:
            shutil.rmtree(parent, ignore_errors=True)
    except Stop as error:
        print(error, file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
