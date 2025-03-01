from uuid import uuid4

from fastapi.testclient import TestClient

from app.schemas.translation import (
    TranslationResponse,
)

pytest_plugins = ["test.fixtures.client"]


def test_create_translation(client: TestClient):
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
