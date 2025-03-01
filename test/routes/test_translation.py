from uuid import UUID, uuid4

from fastapi import status
from fastapi.testclient import TestClient

pytest_plugins = ["test.fixtures.client"]


def test_create_translation_success(client: TestClient):
    """Test creating a single translation successfully"""
    response = client.post(
        "/translations/",
        json={
            "translationKey": "greeting",
            "language": "en",
            "translation": "Hello",
        },
    )
    data = response.json()

    assert response.status_code == status.HTTP_201_CREATED
    assert data["translationKey"] == "greeting"
    assert data["language"] == "en"
    assert data["translation"] == "Hello"


def test_create_bulk_translations_success(client: TestClient):
    """Test creating multiple translations in bulk"""
    response = client.post(
        "/translations/bulk/",
        json={
            "translationKey": "greeting",
            "translations": [
                {
                    "language": "en",
                    "translation": "Hello",
                },
                {
                    "language": "nl",
                    "translation": "Hallo",
                },
                {
                    "language": "fr",
                    "translation": "Bonjour",
                },
            ],
        },
    )
    data = response.json()
    print(data)

    assert response.status_code == status.HTTP_201_CREATED
    assert data["translationKey"] == "greeting"
    assert {t["language"] for t in data["translations"]} == {
        "en",
        "nl",
        "fr",
    }
    assert {t["translation"] for t in data["translations"]} == {
        "Hello",
        "Hallo",
        "Bonjour",
    }


def test_get_translations_success(client: TestClient):
    """Test retrieving translations by key"""
    # First create some translations
    client.post(
        "/translations/bulk/",
        json={
            "translationKey": "greeting",
            "translations": [
                {"language": "en", "translation": "Hello"},
                {"language": "nl", "translation": "Hallo"},
            ],
        },
    )

    response = client.get("/translations/greeting/")
    data = response.json()

    assert response.status_code == status.HTTP_201_CREATED
    assert data["translationKey"] == "greeting"
    assert {t["translation"] for t in data["translations"]} == {
        "Hello",
        "Hallo",
    }
    assert {t["language"] for t in data["translations"]} == {
        "en",
        "nl",
    }


def test_create_duplicate_translation(client: TestClient):
    """Test creating duplicate translation (same key + language)"""
    # Create initial translation
    client.post(
        "/translations/",
        json={
            "translationKey": "greeting",
            "language": "en",
            "translation": "Hello",
        },
    )

    # Try to create duplicate
    response = client.post(
        "/translations/",
        json={
            "translationKey": "greeting",
            "language": "en",
            "translation": "Hi",
        },
    )

    assert response.status_code == status.HTTP_409_CONFLICT


def test_create_bulk_with_duplicates(client: TestClient):
    """Test bulk create with existing translations"""
    # Create initial translations
    client.post(
        "/translations/bulk/",
        json={
            "translationKey": "greeting",
            "translations": [{"language": "en", "translation": "Hello"}],
        },
    )

    # Try bulk create with duplicate
    response = client.post(
        "/translations/bulk/",
        json={
            "translationKey": "greeting",
            "translations": [
                {"language": "en", "translation": "Hi"},
                {"language": "nl", "translation": "Hallo"},
            ],
        },
    )

    assert response.status_code == status.HTTP_409_CONFLICT


def test_get_nonexistent_translations(client: TestClient):
    """Test retrieving translations for non-existent key"""
    response = client.get("/translations/nonexistent/")

    assert response.status_code == status.HTTP_404_NOT_FOUND


def test_auto_generated_key(client: TestClient):
    """Test translation creation with auto-generated UUID key"""
    payload = {"language": "en", "translation": "Hello"}

    response = client.post("/translations/", json=payload)
    data = response.json()

    assert response.status_code == status.HTTP_201_CREATED
    assert data["translationKey"] is not None
    UUID(data["translationKey"], version=4)


def test_invalid_language(client: TestClient):
    """Test creating translation with invalid language"""
    response = client.post("/translations/", json={
        "translationKey": "greeting",
        "language": "xx",
        "translation": "Hello",
    })
    errors = response.json()["detail"]

    assert response.status_code == status.HTTP_422_UNPROCESSABLE_ENTITY
    assert any(e["loc"] == ["body", "language"] for e in errors)
