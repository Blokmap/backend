import jwt
from fastapi.testclient import TestClient

from app.main import app
from app.constants import JWT_ALGORITHM, JWT_SECRET_KEY


client = TestClient(app)


def test_signup():
    response = client.post(
        "/auth/signup",
        data={"username": "bob", "email": "bob@example.com", "password": "appel"},
    )
    data = response.json()

    assert response.status_code == 201
    assert data["id"] is not None
    assert data["username"] == "bob"
    assert data["email"] == "bob@example.com"
    assert data["hashed_password"] is not None


def test_login():
    response = client.post(
        "/auth/login",
        data={"username": "bob", "password": "appel"},
    )
    data = response.json()

    assert response.status_code == 201
    assert data["token_type"] == "bearer"
    assert data["access_token"] is not None

    payload = jwt.decode(
        data["access_token"], JWT_SECRET_KEY, algorithms=[JWT_ALGORITHM]
    )
    id = payload.get("sub")

    assert id is not None
