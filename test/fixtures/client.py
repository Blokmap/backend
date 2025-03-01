import pytest
from fastapi.testclient import TestClient
from sqlmodel import Session

from app.main import app
from app.dependencies.database import get_session


pytest_plugins = ["test.fixtures.session"]


@pytest.fixture(name="client")
def client_fixture(session: Session):
    def get_session_override():
        return session

    app.dependency_overrides[get_session] = get_session_override

    client = TestClient(app)
    yield client

    app.dependency_overrides.clear()
