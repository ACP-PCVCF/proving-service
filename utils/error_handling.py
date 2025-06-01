import logging
from pyzeebe import Job, JobController

logger = logging.getLogger("camunda_service")

async def on_error(exception: Exception, job: Job, job_controller: JobController):
    """
    Error handler for Zeebe worker tasks.
    
    Args:
        exception: The exception that was raised
        job: The job that failed
        job_controller: Controller to manage the job status
    """
    error_message = f"Failed to handle job {job}. Error: {str(exception)}"
    logger.error(error_message, exc_info=True)
    await job_controller.set_error_status(job, error_message)

class ServiceError(Exception):
    """Base exception class for service errors."""
    def __init__(self, message: str, service_name: str):
        self.service_name = service_name
        super().__init__(f"{service_name}: {message}")

class SensorDataServiceError(ServiceError):
    """Exception for errors in the sensor data service."""
    def __init__(self, message: str):
        super().__init__(message, "SensorDataService")

class ProofingServiceError(ServiceError):
    """Exception for errors in the proofing service."""
    def __init__(self, message: str):
        super().__init__(message, "ProofingService")

class CertificateServiceError(ServiceError):
    """Exception for errors in the certificate service."""
    def __init__(self, message: str):
        super().__init__(message, "CertifciationService")