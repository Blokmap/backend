from typing import Self

from pydantic import BaseModel
from sqlmodel import Field, SQLModel, Session, select


class User(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    username: str = Field(index=True)
    email: str = Field(unique=True)
    hashed_password: str = Field(exclude=True)

    @staticmethod
    def get(session: Session, id: int) -> Self | None:
        return session.get(User, id)

    @staticmethod
    def get_by_username(session: Session, username: str) -> Self | None:
        return session.exec(
            select(User).where(User.username == username).limit(1)
        ).first()


class NewUser(BaseModel):
    username: str
    email: str
    password: str
