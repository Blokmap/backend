from fastapi import APIRouter

from app.deps.auth import UserSessionDep


router = APIRouter(prefix="/user")


@router.get("/me")
async def get_current_user(current_user: UserSessionDep):
    return current_user
