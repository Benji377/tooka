import pytest
from typer.testing import CliRunner
from tooka.cli import app

@pytest.fixture
def runner():
    return CliRunner()

@pytest.mark.parametrize(
    "args, expected",
    [
        (["run"], ["name=all", "once=False"]),
        (["run", "--name", "ping", "--once"], ["name=ping", "once=True"]),
    ],
)
def test_run_command(runner, args, expected):
    result = runner.invoke(app, args)
    assert result.exit_code == 0
    for expect in expected:
        assert expect in result.output


@pytest.mark.parametrize(
    "args, expected",
    [
        (["list"], ["name=", "active=False"]),
        (["list", "--name", "job", "--active"], ["name=job", "active=True"]),
    ],
)
def test_list_command(runner, args, expected):
    result = runner.invoke(app, args)
    assert result.exit_code == 0
    for expect in expected:
        assert expect in result.output


def test_add_command(runner):
    result = runner.invoke(app, ["add", "--name", "hello", "--interval", "30", "--command", "mod:func"])
    assert result.exit_code == 0
    assert "name=hello" in result.output
    assert "interval=30" in result.output
    assert "command=mod:func" in result.output


def test_remove_command(runner):
    result = runner.invoke(app, ["remove", "hello"])
    assert result.exit_code == 0
    assert "name=hello" in result.output


@pytest.mark.parametrize(
    "args, expected",
    [
        (["init"], "overwrite=False"),
        (["init", "--overwrite"], "overwrite=True"),
    ]
)
def test_init_command(runner, args, expected):
    result = runner.invoke(app, args)
    assert result.exit_code == 0
    assert expected in result.output


def test_status_command(runner):
    result = runner.invoke(app, ["status"])
    assert result.exit_code == 0
    assert "Called 'status'" in result.output


@pytest.mark.parametrize(
    "args, expected",
    [
        (["validate", "tasks.json"], ["file=tasks.json", "verbose=False"]),
        (["validate", "tasks.json", "--verbose"], ["file=tasks.json", "verbose=True"]),
    ],
)
def test_validate_command(runner, args, expected):
    result = runner.invoke(app, args)
    assert result.exit_code == 0
    for expect in expected:
        assert expect in result.output


def test_version_command(runner, monkeypatch):
    monkeypatch.setattr("importlib.metadata.version", lambda _: "0.1.0")
    result = runner.invoke(app, ["version"])
    assert result.exit_code == 0
    assert "Tooka v0.1.0" in result.output
