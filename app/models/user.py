from typing import Self

from sqlmodel import Field, SQLModel, Session, select


class User(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    username: str = Field(index=True)
    email: str = Field(unique=True)
    hashed_password: str = Field(exclude=True)

    def save(self, session: Session) -> Self:
        session.add(self)
        session.commit()
        session.refresh(self)

        return self

    @staticmethod
    def get(session: Session, id: int) -> Self | None:
        return session.get(User, id)

    @staticmethod
    def get_by_username(session: Session, username: str) -> Self | None:
        return session.exec(
            select(User).where(User.username == username).limit(1)
        ).first()
