from fastapi import APIRouter, status

import app.services.translation as trans_service
from app.dependencies.database import DbSessionDep
from app.schemas.translation import (
    TranslationCreate,
    TranslationResponse,
    TranslationsCreate,
    TranslationsResponse,
)

router = APIRouter(prefix="/translations")


@router.post(
    path="/",
    status_code=status.HTTP_201_CREATED,
    response_model=TranslationResponse,
)
async def create_translation(
    session: DbSessionDep,
    translation: TranslationCreate,
):
    # Create the translation.
    _, translation = trans_service.create_translation(session, translation)

    # Return the translation in the response model.
    return TranslationResponse(**translation.__dict__)


@router.post(
    path="/bulk/",
    status_code=status.HTTP_201_CREATED,
    response_model=TranslationsResponse,
)
async def create_translations(
    session: DbSessionDep,
    translations: TranslationsCreate,
):
    # Create the translations.
    key, translations = trans_service.create_translations(
        session,
        translations,
    )

    # Return the translations in the response model.
    return TranslationsResponse(
        translation_key=key,
        translations=[
            TranslationResponse(**translation.__dict__)
            for translation in translations
        ],
    )


@router.get(
    path="/{key}/",
    status_code=status.HTTP_201_CREATED,
    response_model=TranslationsResponse,
)
async def get_translations(session: DbSessionDep, key: str):
    # Get the translations.
    translations = trans_service.get_translations(session, key)

    # Return the translations in the response model.
    return TranslationsResponse(
        translation_key=key,
        translations=[
            TranslationResponse(**translation.__dict__)
            for translation in translations
        ],
    )
