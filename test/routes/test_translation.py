from uuid import uuid4
from fastapi.testclient import TestClient
from app.main import app
from app.schemas.translation import (
    TranslationResponse,
    TranslationsResponse,
)

client = TestClient(app)


def test_create_translation():
    """Test translation creation."""
    test_key = uuid4()
    payload = {
        "language": "en",
        "translation": "Hello World",
        "translation_key": str(test_key),
    }

    # Test successful creation
    response = client.post("/translation/", json=payload)
    assert response.status_code == 201
    TranslationResponse.model_validate(response.json())

    # Test validation error
    bad_payload = {"language": "en", "translation": ""}
    response = client.post("/translation/", json=bad_payload)
    assert response.status_code == 422


def test_get_translations():
    """Test translation retrieval."""
    test_key = uuid4()

    # Create test translations
    languages = ["en", "es", "fr"]

    for lang in languages:
        client.post(
            "/translation/",
            json={
                "language": lang,
                "translation": f"Translation {lang}",
                "translation_key": str(test_key),
            },
        )

    # Test successful retrieval
    response = client.get(f"/translation/{test_key}/")
    assert response.status_code == 200
    data = TranslationsResponse.model_validate(response.json())

    assert str(data.translation_key) == str(test_key)
    assert len(data.translations) == 3

    for lang in languages:
        assert lang in data.translations
        assert data.translations[lang].translation == f"Translation {lang}"

    # Test non-existent key
    response = client.get(f"/translation/{uuid4()}/")
    assert response.status_code == 200
    data = TranslationsResponse.model_validate(response.json())
    assert len(data.translations) == 0


def test_duplicate_language_protection():
    """Test that duplicate translations are not allowed."""
    test_key = uuid4()
    payload = {
        "language": "de",
        "translation": "Initial DE",
        "translation_key": str(test_key),
    }

    # First creation should succeed
    response = client.post("/translation/", json=payload)
    assert response.status_code == 201

    # Duplicate creation should fail
    response = client.post("/translation/", json=payload)
    assert response.status_code == 400
