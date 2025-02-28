from datetime import datetime
from enum import Enum
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel


class Language(Enum):
    NL = "nl"
    EN = "en"
    FR = "fr"
    DE = "de"


class Translation(SQLModel, table=True):
    id: Optional[int] = Field(primary_key=True)
    language: Language = Field()
    translation_key: UUID = Field()
    translation: str = Field()

    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(
        default_factory=datetime.now,
        sa_column_kwargs={"onupdate": datetime.now},
    )
