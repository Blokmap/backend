from datetime import datetime
from typing import Optional

from pydantic import Field

from app.models.translation import LanguageEnum
from app.schemas import BaseModel


class TranslationBase(BaseModel):
    """Base model for translation data"""

    language: LanguageEnum = Field(...)
    translation_key: Optional[str] = Field(None)
    translation: str = Field(..., min_length=1)


class TranslationCreate(TranslationBase):
    """Request model for creating a new translation"""


class TranslationsCreate(BaseModel):
    """Request model for creating a new translation"""

    translation_key: Optional[str] = Field(...)
    translations: list[TranslationCreate] = Field(...)


class TranslationResponse(TranslationBase):
    """Response model for a single translation"""

    created_at: datetime = Field(...)
    updated_at: datetime = Field(...)


class TranslationsResponse(BaseModel):
    """Response model containing multiple translations keyed by language"""

    translation_key: Optional[str] = Field(...)
    translations: list[TranslationResponse] = Field(...)
