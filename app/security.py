from passlib.context import CryptContext

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


def verify_user_password(plain_password: str, hashed_password: str) -> bool:
    """
    Verifies the given plain text password against a hashed password.

    Args:
        plain_password (str): The plain text password to verify.
        hashed_password (str): The hashed password to verify against.

    Returns:
        bool: True if the password matches, False otherwise.
    """
    return pwd_context.verify(plain_password, hashed_password)
