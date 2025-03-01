import app.services.translation as trans_service

from app.deps.db import DbSessionDep
from app.schemas.translation import (
    TranslationResponse,
    TranslationsCreate,
    TranslationsResponse,
)

from fastapi import APIRouter, status

router = APIRouter(prefix="/translation")


@router.post(
    path="/create/",
    status_code=status.HTTP_201_CREATED,
    response_model=TranslationResponse,
)
async def create_translation(
    session: DbSessionDep,
    translation_data: TranslationsCreate = None,
):
    translation = trans_service.create_translation(
        session,
        translation_data,
        translation_data.translation_key,
    )

    return TranslationResponse(
        **translation.__dict__,
    )

@router.post()


@router
@router.get(
    path="/{key}/",
    status_code=status.HTTP_201_CREATED,
    response_model=TranslationsResponse,
)
async def get_translations(session: DbSessionDep, key: str):
    translations = trans_service.get_translations(session, key)

    return TranslationsResponse(
        translation_key=key,
        translations={
            translation.language: translation for translation in translations
        },
    )
