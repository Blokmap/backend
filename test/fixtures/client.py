import pytest
from fastapi.testclient import TestClient
from sqlmodel import Session

from app.dependencies.database import get_session
from app.main import app

pytest_plugins = ["test.fixtures.session"]


@pytest.fixture(name="client")
def client_fixture(session: Session):
    app.dependency_overrides[get_session] = lambda: session

    client = TestClient(app)
    yield client

    app.dependency_overrides.clear()
