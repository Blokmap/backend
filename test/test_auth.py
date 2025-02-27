from fastapi.testclient import TestClient

from app.main import app


client = TestClient(app)


def test_signup():
    response = client.post(
        "/auth/signup",
        json={"username": "bob", "email": "bob@example.com", "password": "appel"},
    )
    data = response.json()

    assert response.status_code == 201
    assert data["id"] is not None
    assert data["username"] == "bob"
    assert data["email"] == "bob@example.com"
    assert data["hashed_password"] is not None
