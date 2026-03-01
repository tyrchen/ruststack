# Standalone conftest.py for running ScyllaDB Alternator tests against
# RustStack or any DynamoDB-compatible endpoint.
#
# Replaces the ScyllaDB-internal conftest.py with one that uses fixed
# credentials and connects to a configurable URL (default: localhost:4566).

import pytest
import boto3
import botocore

from util import create_test_table


# ---------------------------------------------------------------------------
# pytest CLI options
# ---------------------------------------------------------------------------


def pytest_addoption(parser):
    parser.addoption(
        "--url",
        action="store",
        default="http://localhost:4566",
        help="DynamoDB-compatible endpoint URL (default: http://localhost:4566)",
    )
    parser.addoption(
        "--aws",
        action="store_true",
        default=False,
        help="Run against real AWS DynamoDB (requires ~/.aws/credentials)",
    )
    parser.addoption(
        "--https",
        action="store_true",
        default=False,
        help="Use HTTPS (unused for RustStack, kept for compatibility)",
    )
    parser.addoption(
        "--runveryslow",
        action="store_true",
        default=False,
        help="Run tests marked veryslow",
    )


# ---------------------------------------------------------------------------
# Skip markers
# ---------------------------------------------------------------------------


def pytest_configure(config):
    config.addinivalue_line("markers", "veryslow: mark test as very slow")
    config.addinivalue_line("markers", "scylla_only: mark test as ScyllaDB-specific")


def pytest_collection_modifyitems(config, items):
    """Skip tests that are only relevant to ScyllaDB or require special flags."""
    skip_veryslow = pytest.mark.skip(reason="need --runveryslow to run")
    skip_scylla = pytest.mark.skip(reason="ScyllaDB-specific test")

    for item in items:
        if "veryslow" in item.keywords and not config.getoption("--runveryslow"):
            item.add_marker(skip_veryslow)
        if "scylla_only" in item.keywords:
            item.add_marker(skip_scylla)


# ---------------------------------------------------------------------------
# Core fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="session")
def dynamodb(request):
    """Provide a boto3 DynamoDB resource connected to the target endpoint."""
    boto_config = botocore.client.Config(
        parameter_validation=False,
        retries={"max_attempts": 0},
        read_timeout=300,
    )

    if request.config.getoption("aws"):
        res = boto3.resource("dynamodb", config=boto_config)
    else:
        url = request.config.getoption("url")
        res = boto3.resource(
            "dynamodb",
            endpoint_url=url,
            region_name="us-east-1",
            aws_access_key_id="test",
            aws_secret_access_key="test",
            config=boto_config,
        )

    yield res
    res.meta.client.close()


@pytest.fixture(scope="session")
def dynamodbstreams(request):
    """Provide a boto3 DynamoDB Streams client (stub for non-stream endpoints)."""
    boto_config = botocore.client.Config(
        parameter_validation=False,
        retries={"max_attempts": 0},
        read_timeout=300,
    )

    if request.config.getoption("aws"):
        client = boto3.client("dynamodbstreams", config=boto_config)
    else:
        url = request.config.getoption("url")
        client = boto3.client(
            "dynamodbstreams",
            endpoint_url=url,
            region_name="us-east-1",
            aws_access_key_id="test",
            aws_secret_access_key="test",
            config=boto_config,
        )

    yield client
    client.close()


# ---------------------------------------------------------------------------
# ScyllaDB-specific fixture stubs
# ---------------------------------------------------------------------------


@pytest.fixture(scope="session")
def rest_api():
    """Stub: ScyllaDB REST API fixture. Always skips."""
    pytest.skip("ScyllaDB REST API not available")


@pytest.fixture(scope="session")
def cql():
    """Stub: CQL session fixture. Always skips."""
    pytest.skip("CQL not available on non-ScyllaDB endpoint")


@pytest.fixture
def scylla_only():
    """Stub: skip ScyllaDB-only tests."""
    pytest.skip("ScyllaDB-specific test")


@pytest.fixture(scope="session")
def has_tablets():
    """Stub: tablet feature detection. Always False."""
    return False


# ---------------------------------------------------------------------------
# Table fixtures (match ScyllaDB Alternator conftest.py API)
# ---------------------------------------------------------------------------


@pytest.fixture(scope="session")
def test_table(dynamodb):
    """Table with composite key: p (HASH, S) + c (RANGE, S)."""
    table = create_test_table(
        dynamodb,
        KeySchema=[
            {"AttributeName": "p", "KeyType": "HASH"},
            {"AttributeName": "c", "KeyType": "RANGE"},
        ],
        AttributeDefinitions=[
            {"AttributeName": "p", "AttributeType": "S"},
            {"AttributeName": "c", "AttributeType": "S"},
        ],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_s(dynamodb):
    """Table with simple key: p (HASH, S)."""
    table = create_test_table(
        dynamodb,
        KeySchema=[{"AttributeName": "p", "KeyType": "HASH"}],
        AttributeDefinitions=[{"AttributeName": "p", "AttributeType": "S"}],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_s_2(dynamodb):
    """Second table with simple key: p (HASH, S) for multi-table tests."""
    table = create_test_table(
        dynamodb,
        KeySchema=[{"AttributeName": "p", "KeyType": "HASH"}],
        AttributeDefinitions=[{"AttributeName": "p", "AttributeType": "S"}],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_b(dynamodb):
    """Table with binary key: p (HASH, B)."""
    table = create_test_table(
        dynamodb,
        KeySchema=[{"AttributeName": "p", "KeyType": "HASH"}],
        AttributeDefinitions=[{"AttributeName": "p", "AttributeType": "B"}],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_sb(dynamodb):
    """Table with composite key: p (HASH, S) + c (RANGE, B)."""
    table = create_test_table(
        dynamodb,
        KeySchema=[
            {"AttributeName": "p", "KeyType": "HASH"},
            {"AttributeName": "c", "KeyType": "RANGE"},
        ],
        AttributeDefinitions=[
            {"AttributeName": "p", "AttributeType": "S"},
            {"AttributeName": "c", "AttributeType": "B"},
        ],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_sn(dynamodb):
    """Table with composite key: p (HASH, S) + c (RANGE, N)."""
    table = create_test_table(
        dynamodb,
        KeySchema=[
            {"AttributeName": "p", "KeyType": "HASH"},
            {"AttributeName": "c", "KeyType": "RANGE"},
        ],
        AttributeDefinitions=[
            {"AttributeName": "p", "AttributeType": "S"},
            {"AttributeName": "c", "AttributeType": "N"},
        ],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def test_table_ss(dynamodb):
    """Table with composite key: p (HASH, S) + c (RANGE, S). Alias for test_table."""
    table = create_test_table(
        dynamodb,
        KeySchema=[
            {"AttributeName": "p", "KeyType": "HASH"},
            {"AttributeName": "c", "KeyType": "RANGE"},
        ],
        AttributeDefinitions=[
            {"AttributeName": "p", "AttributeType": "S"},
            {"AttributeName": "c", "AttributeType": "S"},
        ],
    )
    yield table
    table.delete()


@pytest.fixture(scope="session")
def filled_test_table(test_table):
    """Pre-populate test_table with 329 items for read-intensive tests.

    Creates items with p='long', c='0000' through c='0328'.
    Each item has additional attribute 'another' with a string value.
    """
    items = []
    for i in range(329):
        item = {"p": "long", "c": f"{i:04d}", "another": f"value_{i}"}
        items.append(item)

    # Use batch writer for efficiency.
    with test_table.batch_writer() as batch:
        for item in items:
            batch.put_item(Item=item)

    yield test_table, items
