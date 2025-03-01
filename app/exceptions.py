from fastapi import status


class BlokmapApiError(Exception):
    """Base exception class"""

    def __init__(self, message: str, status_code: int = 500):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)


class InvalidToken(BlokmapApiError):
    """Exception raised when the token is invalid"""
    def __init__(self, message: str):
        super().__init__(message, status.HTTP_401_UNAUTHORIZED)


class EntityDoesNotExist(BlokmapApiError):
    """Exception raised when an entity does not exist in the database"""
    def __init__(self, message: str):
        super().__init__(message, status.HTTP_404_NOT_FOUND)


class EntityAlreadyExists(BlokmapApiError):
    """Exception raised when an entity already exists in the database"""
    def __init__(self, message: str):
        super().__init__(message, status.HTTP_409_CONFLICT)
