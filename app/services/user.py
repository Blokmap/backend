from sqlmodel import Session, select

from app.models.user import User
from app.schemas.user import UserCreate
from app.security import hash_user_password


def get_user_by_id(session: Session, id: int) -> User | None:
    """
    Get a user by their ID.

    Args:
        session (Session): The database session to use for querying.
        id (int): The ID of the user to get.

    Returns:
        User | None: The user if found, otherwise None.
    """
    return session.get(User, id)


def get_user_by_username(session: Session, username: str) -> User | None:
    """
    Get a user by their username.

    Args:
        session (Session): The database session to use for querying.
        username (str): The username of the user to get.

    Returns:
        User | None: The user if found, otherwise None.
    """
    user = session.exec(select(User).where(User.username == username)).first()
    return user


def create_user(session: Session, user: UserCreate) -> User:
    """
    Create a new user in the database.

    Args:
        session (Session): The database session to use for the transaction.
        user (UserCreate): The user data to create.

    Returns:
        User: The created user object with updated information from the database.
    """
    hashed_password = hash_user_password(user.password)

    user = User(
        username=user.username,
        email=user.email,
        hashed_password=hashed_password,
    )

    session.add(user)
    session.commit()
    session.refresh(user)

    return user
