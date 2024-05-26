import asyncio
from argparse import ArgumentParser

from aiogram import Bot
from aiogram.client.default import DefaultBotProperties
from aiogram.enums import ParseMode
from aiogram.types import FSInputFile

import subprocess
import os

def create_parser() -> ArgumentParser:
    parser = ArgumentParser()
    parser.add_argument("--token", help="Telegram Bot API Token")
    parser.add_argument("--chat-id", type=int, help="Target chat id")

    return parser

async def main():
    parser = create_parser()
    ns = parser.parse_args()

    token = ns.token
    chat_id = ns.chat_id

    async with Bot(
        token=token,
        default=DefaultBotProperties(
            parse_mode=ParseMode.HTML,
        ),
    ) as bot:
        await bot.send_message(chat_id=chat_id, text="Script started")
    output = subprocess.run("./run-benchmarks.sh > logs.txt", shell=True)
    async with Bot(
        token=token,
        default=DefaultBotProperties(
            parse_mode=ParseMode.HTML,
        ),
    ) as bot:
        await bot.send_message(chat_id=chat_id, text=f"Benchmarks completed with status code {output.returncode}")
        try:
            output = FSInputFile(os.path.join(os.path.dirname(__file__), "linode_benchmarks.txt"))
            await bot.send_document(chat_id=chat_id, document=output) 
        except Exception as e:
            await bot.send_message(chat_id=chat_id, text=f"Failed to send results")
            print(e)

        try:
            logs = FSInputFile(os.path.join(os.path.dirname(__file__), "logs.txt"))
            await bot.send_document(chat_id=chat_id, document=logs, caption="Logs") 
        except Exception as e:
            await bot.send_message(chat_id=chat_id, text=f"Failed to send logs")
            print(e)

if __name__ == "__main__":
    asyncio.run(main())
