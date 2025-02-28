from datetime import datetime
from typing import Optional
from uuid import UUID
from app.models.translation import Language
from app.schemas import BaseModel


class NewTranslation(BaseModel):
    language: Language
    translation_key: Optional[UUID] = None
    translation: str

class ResponseTranslation(BaseModel):
    language: Language
    translation_key: Optional[UUID] = None
    translation: str
    created_at: datetime
    updated_at: datetime