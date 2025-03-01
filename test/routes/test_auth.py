import jwt
from fastapi.testclient import TestClient

from app.constants import JWT_ALGORITHM, JWT_SECRET_KEY

pytest_plugins = ["test.fixtures.client"]


def test_signup(client: TestClient):
    response = client.post(
        "/auth/signup",
        data={
            "username": "bob",
            "email": "bob@example.com",
            "password": "appel",
        },
    )
    data = response.json()

    assert response.status_code == 201
    assert data.get("id") is not None
    assert data.get("username") == "bob"
    assert data.get("email") == "bob@example.com"
    assert data.get("hashed_password") is None
    assert response.cookies.get("access_token") is not None

    payload = jwt.decode(
        response.cookies.get("access_token"),
        JWT_SECRET_KEY,
        algorithms=[JWT_ALGORITHM],
    )
    id = payload.get("sub")

    assert id is not None


def test_login(client: TestClient):
    client.post(
        "/auth/signup",
        data={
            "username": "bob",
            "email": "bob@example.com",
            "password": "appel",
        },
    )

    response = client.post(
        "/auth/login",
        data={"username": "bob", "password": "appel"},
    )

    assert response.status_code == 200
    assert response.cookies.get("access_token") is not None

    payload = jwt.decode(
        response.cookies.get("access_token"),
        JWT_SECRET_KEY,
        algorithms=[JWT_ALGORITHM],
    )
    id = payload.get("sub")

    assert id is not None


def test_user_route(client: TestClient):
    client.post(
        "/auth/signup",
        data={
            "username": "bob",
            "email": "bob@example.com",
            "password": "appel",
        },
    )

    response = client.post(
        "/auth/login",
        data={"username": "bob", "password": "appel"},
    )
    access_token = response.cookies.get("access_token")

    client.cookies = {"access_token": access_token}

    response = client.get("/user/me")
    data = response.json()

    assert response.status_code == 200
    assert data.get("id") is not None
    assert data.get("username") == "bob"
    assert data.get("email") == "bob@example.com"
    assert data.get("hashed_password") is None
