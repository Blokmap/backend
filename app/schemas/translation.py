from datetime import datetime
from typing import Optional
from uuid import UUID
from app.schemas import BaseModel
from pydantic import Field, ConfigDict, RootModel
from app.models.translation import Language


class TranslationBase(BaseModel):
    """Base model for translation data"""

    language: Language = Field(...)
    translation: str = Field(
        ...,
        min_length=1,
    )


class TranslationCreate(TranslationBase):
    """Request model for creating a new translation"""

    translation_key: Optional[UUID] = Field(None)


class TranslationResponse(TranslationBase):
    """Response model for a single translation"""

    translation_key: UUID = Field(...)
    created_at: datetime = Field(...)
    updated_at: datetime = Field(...)
    model_config = ConfigDict(from_attributes=True)


class TranslationsResponse(BaseModel):
    """Response model containing multiple translations keyed by language"""

    translation_key: Optional[UUID] = Field(...)
    translations: dict[Language, TranslationResponse] = Field(...)
