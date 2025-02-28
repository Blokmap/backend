from typing import Optional
from app.models.translation import Language
from pydantic import BaseModel


class NewTranslation(BaseModel):
    key: Optional[str]
    language: Language
    content: str
