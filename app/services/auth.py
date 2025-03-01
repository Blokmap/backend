from datetime import datetime, timedelta, timezone

import jwt
from sqlmodel import Session

from app.constants import JWT_ALGORITHM, JWT_SECRET_KEY
from app.models.user import User
from app.security import verify_user_password
from app.services.user import get_user_by_username


def create_access_token(data: dict, expires_delta: timedelta) -> str:
    """
    Creates a JSON Web Token (JWT) for the given data with an expiration time.

    Args:
        data (dict): The data to encode in the JWT.
        expires_delta (timedelta, optional): The time duration after which the token will expire.
                                             Defaults to 15 minutes.

    Returns:
        str: The encoded JWT as a string.
    """
    to_encode = data.copy()

    expire = datetime.now(timezone.utc) + expires_delta
    to_encode.update({"exp": expire})

    encoded_jwt = jwt.encode(to_encode, JWT_SECRET_KEY, algorithm=JWT_ALGORITHM)

    return encoded_jwt


def authenticate_user(
    username: str, password: str, session: Session
) -> User | None:
    """
    Authenticate a user by their username and password.

    Args:
        username (str): The username of the user.
        password (str): The password of the user.
        session (Session): The database session to use for querying.

    Returns:
        User | None: The authenticated user if credentials are valid, otherwise None.
    """
    user = get_user_by_username(session, username)

    if user is not None:
        verified = verify_user_password(password, user.hashed_password)

        if verified:
            return user
