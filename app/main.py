from fastapi import FastAPI


from .model import Todo
from .database import session


app = FastAPI(
    root_path="/api",
    docs_url=None,
    redoc_url=None
)


@app.get("/")
async def get_all_todos():
    todos_query = session.query(Todo)
    return todos_query.all()


@app.post("/create")
async def create_todo(text: str, is_complete: bool = False):
    todo = Todo(text=text, is_done=is_complete)
    session.add(todo)
    session.commit()
    return {"todo added": todo.text}
