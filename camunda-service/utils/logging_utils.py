import logging
import os
import sys

from config.settings import LOG_LEVEL


def setup_logging():
    """Configure the logger for the application."""
    log_formatter = logging.Formatter(
        "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    logger = logging.getLogger("camunda_service")
    logger.setLevel(getattr(logging, LOG_LEVEL))
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(log_formatter)
    logger.addHandler(console_handler)
    
    # File handler
    os.makedirs("logs", exist_ok=True)
    file_handler = logging.FileHandler("logs/camunda_service.log")
    file_handler.setFormatter(log_formatter)
    logger.addHandler(file_handler)
    
    return logger


def log_task_start(task_name, **context):
    """Log the start of a task."""
    logger = logging.getLogger("camunda_service")
    context_str = ", ".join(f"{k}={v}" for k, v in context.items())
    logger.info(f"Starting task: {task_name}" + (f" with {context_str}" if context else ""))


def log_task_completion(task_name, **result):
    """Log the completion of a task."""
    logger = logging.getLogger("camunda_service")
    result_str = ", ".join(f"{k}={v}" for k, v in result.items())
    logger.info(f"Task completed: {task_name}" + (f" with {result_str}" if result else ""))


def log_service_call(service_name, method_name, **context):
    """Log a service method call."""
    logger = logging.getLogger("camunda_service")
    context_str = ", ".join(f"{k}={v}" for k, v in context.items())
    logger.info(f"Service call: {service_name}.{method_name}" + (f" with {context_str}" if context else ""))