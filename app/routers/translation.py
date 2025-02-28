from app.database import get_session
from app.deps.db import DbSessionDep
from app.models.translation import Translation
from app.schemas.translation import NewTranslation, ResponseTranslation
from app.services.translation import create_translation, get_translations
from fastapi import APIRouter, Depends, status
from sqlmodel import Session

router = APIRouter(prefix="/translation")


@router.post(
    "/", status_code=status.HTTP_201_CREATED, response_model=ResponseTranslation
)
async def create_translation_route(
    session: Session = Depends(get_session),
    translation_data: NewTranslation = None,
):
    translation = create_translation(
        session, translation_data, translation_data.translation_key
    )

    return ResponseTranslation(
        language=translation.language,
        translation_key=translation.translation_key,
        translation=translation.translation,
        created_at=translation.created_at,
        updated_at=translation.updated_at,
    )


@router.get("/{key}/")
async def get_translations_route(session: DbSessionDep, key: str):
    return get_translations(session, key)
