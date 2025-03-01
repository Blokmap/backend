from typing import Optional

from sqlmodel import Field, Relationship, SQLModel

from app.models.translation import Translation


class Location(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    name: str = Field()
    latitude: float
    longitude: float

    description_key: str = Field(foreign_key="translation.key")
    description_translations = list[Translation] = Relationship(
        sa_relationship_kwargs={
            "primaryjoin": "Location.description_key == Translation.key",
            "lazy": "joined",
            "viewonly": True,
        }
    )

    excerpt_key: Optional[str] = Field(foreign_key="translation.key")
    excerpt_translations = list[Translation] = Relationship(
        sa_relationship_kwargs={
            "primaryjoin": "Location.excerpt_key == Translation.key",
            "lazy": "joined",
            "viewonly": True,
        }
    )

    @property
    def description(self) -> str:
        return {t.language: t.value for t in self.name_translations}

    @property
    def excerpt(self) -> str:
        return {t.language: t.value for t in self.excerpt_translations}
