from typing import Annotated

from fastapi import Cookie, Depends, HTTPException, status
from jwt import InvalidTokenError, decode

from app.constants import JWT_ALGORITHM, JWT_SECRET_KEY
from app.dependencies.database import DbSessionDep
from app.models.user import User


async def get_user_session(
    access_token: Annotated[str, Cookie()], session: DbSessionDep
):
    """
    Retrieve the user session based on the provided access token.
    Args:
        access_token (Annotated[str, Cookie]): The access token from the user's cookies.
        session (DbSessionDep): The database session dependency.
    Raises:
        HTTPException: If the access token is invalid or the user is not found.
    Returns:
        User: The user object corresponding to the provided access token.
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Invalid username or password",
        headers={"WWW-Authenticate": "Bearer"},
    )

    malformed_token_exception = HTTPException(
        status_code=status.HTTP_400_BAD_REQUEST,
        detail="Could not valid credentials",
    )

    try:
        payload = decode(
            access_token, JWT_SECRET_KEY, algorithms=[JWT_ALGORITHM]
        )
        id = payload.get("sub")

        if id is None:
            raise credentials_exception
    except InvalidTokenError:
        raise malformed_token_exception

    user = session.get(User, int(id))

    if user is None:
        raise credentials_exception

    return user


UserSessionDep = Annotated[User, Depends(get_user_session)]
