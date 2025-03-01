from pydantic import BaseModel, Field


class UserCreate(BaseModel):
    username: str = Field(...)
    email: str = Field(...)
    password: str = Field(...)


class UserResponse(BaseModel):
    username: str = Field(...)
    email: str = Field(...)
    is_active: bool = Field(...)
    created_at: str = Field(...)
    updated_at: str = Field(...)
