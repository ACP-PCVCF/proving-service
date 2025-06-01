# Camunda Service

A Python-based microservice for handling Camunda/Zeebe workflow tasks in a transport carbon emission (TCE) tracking system. This service connects to a Zeebe workflow engine and orchestrates interactions with various external services for sensor data collection, sensor verification, and carbon footprint proofing.

## Overview

This service implements a worker that connects to Camunda 8 Zeebe engine. It acts as an orchestrator that calls various external services including the sensor data service for transportation data, the certification service for sensor verification, and the proofing service to obtain Product Carbon Footprint (PCF) data and proofs.

## Requirements

- Python 3.10+
- Zeebe Server (Camunda 8 Platform)
- Camunda Modeler
- Dependencies listed in `requirements.txt`

## Installation

1. Clone this repository
2. Create a virtual environment (recommended)
   ```
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```
3. Install dependencies
   ```
   pip install -r requirements.txt
   ```

## Configuration

Edit the configuration settings in `config/settings.py` to match your environment:

- `ZEEBE_ADDRESS`: The address of your Zeebe gateway
- `LOG_LEVEL`: Logging level (DEBUG, INFO, WARNING, ERROR)


## Usage

Start the service by running:

```
python main.py
```

The service will:

1. Connect to the configured Zeebe gateway
2. Register task handlers for all workflow tasks
3. Begin processing tasks from the workflow engine

## Development

### Adding New Tasks

To add a new task:

1. Implement the task function in `tasks/worker_tasks.py`
2. Register the task in the `_register_tasks` method
3. Update your BPMN workflow to include the new task

### Testing (to be done)

Run the tests using:

```
python -m unittest discover tests
```
