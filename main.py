"""
Camunda Service - Main entry point
This module initializes and starts the Zeebe worker service for processing Camunda tasks.
"""

import asyncio

from pyzeebe import create_insecure_channel, ZeebeClient, ZeebeWorker

from config.settings import ZEEBE_ADDRESS
from tasks.worker_tasks import CamundaWorkerTasks
from utils.logging_utils import setup_logging


async def main():
    """
    Main entry point for the Camunda Service.
    Sets up the Zeebe client and worker, registers tasks, and starts the worker.
    """
    # Setup logging
    logger = setup_logging()
    logger.info("Starting Camunda Service")

    # Create Zeebe channel and client
    logger.info(f"Connecting to Zeebe at {ZEEBE_ADDRESS}")
    channel = create_insecure_channel(grpc_address=ZEEBE_ADDRESS)
    client = ZeebeClient(channel)
    worker = ZeebeWorker(channel)

    # Initialize worker tasks
    logger.info("Registering worker tasks")
    worker_tasks = CamundaWorkerTasks(worker, client)
    # Start the worker
    logger.info("Starting Zeebe worker")
    try:
        await worker.work()
    except KeyboardInterrupt:
        logger.info("Received keyboard interrupt, shutting down")
    except Exception as e:
        logger.error(f"Error in worker: {e}", exc_info=True)
    finally:
        logger.info("Closing Zeebe connections")
        await channel.close()


if __name__ == "__main__":
    asyncio.run(main())
