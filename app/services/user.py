from app.models.user import User
from app.schemas.user import UserCreate
from app.security import hash_user_password
from sqlmodel import Session, select


def get_user_by_id(session: Session, id: int) -> User | None:
    """Get a user by their ID."""
    return session.get(User, id)


def get_user_by_username(session: Session, username: str) -> User | None:
    """Get a user by their username."""
    user = session.exec(select(User).where(User.username == username)).first()
    return user


def create_user(session: Session, user: UserCreate) -> User:
    """Create a new user."""
    hashed_password = hash_user_password(user.password)

    user = User(
        username=user.username, email=user.email, hashed_password=hashed_password
    )

    session.add(user)
    session.commit()
    session.refresh(user)

    return user
