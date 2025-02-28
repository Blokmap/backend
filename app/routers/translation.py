from app.deps.db import DbSessionDep
from app.services.translation import create_translation, get_translations
from fastapi import APIRouter

router = APIRouter(prefix="/translation")


@router.post("/{key}/")
async def create_translation_route(
    session: DbSessionDep, key: str
):
    return create_translation(session, key)


@router.get("/{key}/")
async def get_translations_route(
    session: DbSessionDep, key: str
):
    return get_translations(session, key)