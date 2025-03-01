from app.deps.db import DbSessionDep
from app.schemas.translation import (
    TranslationCreate,
    TranslationResponse,
    TranslationsResponse,
)
from app.services.translation import create_translation, get_translations
from fastapi import APIRouter, status

router = APIRouter(prefix="/translation")


@router.post(
    "/", status_code=status.HTTP_201_CREATED, response_model=TranslationResponse
)
async def create_translation_rte(
    session: DbSessionDep,
    translation_data: TranslationCreate = None,
):
    translation = create_translation(
        session, translation_data, translation_data.translation_key
    )

    return TranslationResponse(
        **translation.__dict__,
    )


@router.get(
    "/{key}/", status_code=status.HTTP_201_CREATED, response_model=TranslationsResponse
)
async def get_translations_rte(session: DbSessionDep, key: str):
    translations = get_translations(session, key)

    return TranslationsResponse(
        translation_key=key,
        translations={
            translation.language: translation
            for translation in translations
        }
    )
