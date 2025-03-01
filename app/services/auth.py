import jwt
from datetime import datetime, timedelta, timezone

from passlib.context import CryptContext
from sqlmodel import Session, select

from app.constants import ACCESS_TOKEN_EXPIRE_MINUTES, JWT_ALGORITHM, JWT_SECRET_KEY
from app.models.user import User


pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def hash_user_password(password: str) -> str:
    """
    Hashes the given user password using a secure hashing algorithm.

    Args:
        password (str): The plain text password to be hashed.

    Returns:
        str: The hashed password.
    """
    return pwd_context.hash(password)


def create_access_token(
    data: dict, expires_delta: timedelta = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
) -> str:
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


def authenticate_user(username: str, password: str, session: Session) -> User | None:
    """
    Authenticate a user by their username and password.

    Args:
        username (str): The username of the user.
        password (str): The password of the user.
        session (Session): The database session to use for querying.

    Returns:
        User | None: The authenticated user if credentials are valid, otherwise None.
    """
    user = session.exec(select(User).where(User.username == username)).first()

    if user is not None:
        verified = pwd_context.verify(password, user.hashed_password)

        if verified:
            return user
