import logging
import os
from pathlib import Path
import re
import shutil
import sys
from typing import cast, Optional
import pytest
from rmskin_builder import main

FOOTER_LEN = 16


def test_run(
    caplog: pytest.LogCaptureFixture, monkeypatch: pytest.MonkeyPatch, tmpdir: Path
):
    shutil.copytree(
        str(Path(__file__).parent / "demo_project"), str(tmpdir), dirs_exist_ok=True
    )
    monkeypatch.chdir(str(tmpdir))
    if "GITHUB_OUTPUT" not in os.environ:
        gh_out = Path("github_fake_output.txt")
        gh_out.write_bytes(b"")
        monkeypatch.setenv("GITHUB_OUTPUT", str(gh_out.resolve()))
    else:
        gh_out = Path(os.environ["GITHUB_OUTPUT"])
    monkeypatch.setenv("ACTIONS_STEP_DEBUG", "true")

    caplog.set_level(logging.DEBUG)
    monkeypatch.setattr(sys, "argv", [])
    main()

    for line in gh_out.read_text(encoding="utf-8").splitlines():
        if line.startswith("arc-name="):
            arc_name = line.split("=")[1]
            break
    else:
        raise RuntimeError("arc_name output variable not found!")

    archive = Path(tmpdir / arc_name)
    assert archive.exists()

    logged_footer = ""
    logged_size: Optional[int] = None
    for record in caplog.records:
        msg = cast(logging.LogRecord, record).message
        if msg.startswith("Appending footer: "):
            logged_footer = msg[18:]
        if msg.startswith("Archive size = "):
            match = re.match(r"(\d+) \(0x[0-9A-Fa-f]+\)", msg[15:])
            assert match is not None
            logged_size = int(match.groups()[0]) + FOOTER_LEN

    assert logged_footer
    assert logged_size is not None

    assert archive.stat().st_size == logged_size

    footer = ", ".join(["%X" % x for x in archive.read_bytes()[-FOOTER_LEN:]])
    assert f"[{footer}]" == logged_footer
