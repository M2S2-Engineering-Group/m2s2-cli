from fastapi import APIRouter

router = APIRouter()


@router.get("/info")
async def info():
    return {"service": "api", "version": "1.0.0"}
