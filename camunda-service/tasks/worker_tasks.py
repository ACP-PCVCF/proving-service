import random
import uuid
import datetime
from typing import Optional

from pyzeebe import ZeebeWorker, ZeebeClient, Job
from services.database import HocTocService
from utils.error_handling import on_error
from utils.logging_utils import log_task_start, log_task_completion

from services.service_implementations.service_sensordata import SensorDataService
from models.product_footprint import ProductFootprint, Extension, ExtensionData, TceData, Distance


class CamundaWorkerTasks:
    """Zeebe worker task handlers."""

    def __init__(self, worker: ZeebeWorker, client: ZeebeClient):
        self.worker = worker
        self.client = client
        self.hoc_toc_service = HocTocService()
        self.sensor_data_service = SensorDataService()

        # Register all tasks
        self._register_tasks()

    def _register_tasks(self):
        """Register all task handlers with the Zeebe worker."""
        self.worker.task(task_type="determine_job_sequence",
                         exception_handler=on_error)(self.determine_job_sequence)
        self.worker.task(task_type="send_to_proofing_service",
                         exception_handler=on_error)(self.send_to_proofing_service)
        self.worker.task(task_type="notify_next_node",
                         exception_handler=on_error)(self.notify_next_node)
        self.worker.task(task_type="send_data_to_origin",
                         exception_handler=on_error)(self.send_data_to_origin)
        self.worker.task(task_type="define_product_footprint_template",
                         exception_handler=on_error)(self.define_product_footprint_template)
        self.worker.task(task_type="hub_procedure",
                         exception_handler=on_error)(self.hub_procedure)
        self.worker.task(task_type="transport_procedure",
                         exception_handler=on_error)(self.transport_procedure)
        self.worker.task(task_type="set_shipment_information",
                         exception_handler=on_error)(self.set_shipment_information)
        self.worker.task(task_type="collect_hoc_toc_data",
                         exception_handler=on_error)(self.collect_hoc_toc_data)

    def collect_hoc_toc_data(self, product_footprint: dict, sensor_data: Optional[list[dict]] = None) -> dict:
        """
        Collect HOC and TOC data based on product footprint.
        Args:
            product_footprint: Product footprint data
            sensor_data: Optional sensor data to include in the proofing document
        Returns:
            Dictionary containing the proofing document with HOC and TOC data
        """

        log_task_start("collect_hoc_toc_data")
        result = self.hoc_toc_service.collect_hoc_toc_data(product_footprint, sensor_data)
        log_task_completion("collect_hoc_toc_data")

        return result

    def transport_procedure(self, tocId: int, product_footprint: dict, job: Job, sensor_data: Optional[list[dict]] = None) -> dict:
        """
        Handle the hub procedure for a given tocId and product footprint.

        Args:
            tocId: Unique identifier for the transport operation category (toc)
            job: Zeebe Job instance containing process instance and element ID
            product_footprint: Product footprint data
            sensor_data: Optional list of previous sensor data dictionaries to append to

        Returns:
            product_footprint with tocId Information
        """

        log_task_start("transport_procedure")
        new_tce_id = str(uuid.uuid4())

        process_id = job.process_instance_key
        print(f"Received job for process instance: {process_id}")
        element_id = job.element_id
        print(f"Element ID (from BPMN diagram): {element_id}")

        product_footprint_verified = ProductFootprint.model_validate(
            product_footprint)
        # call greta with TceSensorData object, filled with new_tce_id, camunda Process Instance Key and camunda Activity Id
        # receive instance of TceSensorData back
        new_sensor_data = self.sensor_data_service.call_service_sensordata({
            "shipment_id": product_footprint_verified.extensions[0].data.shipmentId,
            "tceId": new_tce_id,
            "camundaProcessInstanceKey": str(process_id),
            "camundaActivityId": element_id
        })
        if sensor_data is not None:
            sensor_data.append(new_sensor_data.model_dump())
        else:
            sensor_data = [new_sensor_data.model_dump()]

        distance_from_sensor = new_sensor_data.sensorData.distance.actual
        #distance_from_sensor = random.uniform(10, 1000)

        prev_tce_ids = []

        if len(product_footprint_verified.extensions[0].data.tces) > 0:
            prev_tce_ids = product_footprint_verified.extensions[0].data.tces[-1].prevTceIds.copy(
            )
            last_tceid = product_footprint_verified.extensions[0].data.tces[-1].tceId

            prev_tce_ids.append(last_tceid)

        new_TCE = TceData(
            tceId=new_tce_id,
            shipmentId=product_footprint_verified.extensions[0].data.shipmentId,
            mass=product_footprint_verified.extensions[0].data.mass,
            distance=Distance(
                actual=distance_from_sensor
            ),
            tocId=tocId,
            prevTceIds=prev_tce_ids
        )

        product_footprint_verified.extensions[0].data.tces.append(
            new_TCE
        )

        result = {
            "product_footprint": product_footprint_verified.model_dump(),
            "sensor_data": sensor_data
        }
        log_task_completion("transport_procedure")

        return result

    def hub_procedure(self, hocId: str, product_footprint: dict) -> dict:
        """
        Handle the hub procedure for a given hocId and product footprint.

        Args:
            hocId: Unique identifier for the hub operation category (hoc)
            product_footprint: Product footprint data

        Returns:
            product_footprint with hocId Information
        """

        log_task_start("hub_procedure")

        product_footprint_verified = ProductFootprint.model_validate(
            product_footprint)

        prev_tce_ids = []
        if len(product_footprint_verified.extensions[0].data.tces) > 0:
            prev_tce_ids = product_footprint_verified.extensions[0].data.tces[-1].prevTceIds.copy(
            )
            last_tceid = product_footprint_verified.extensions[0].data.tces[-1].tceId

            prev_tce_ids.append(last_tceid)

        new_TCE = TceData(
            tceId=str(uuid.uuid4()),
            shipmentId=product_footprint_verified.extensions[0].data.shipmentId,
            mass=product_footprint_verified.extensions[0].data.mass,
            hocId=hocId,
            prevTceIds=prev_tce_ids
        )
        product_footprint_verified.extensions[0].data.tces.append(
            new_TCE
        )
        result = {
            "product_footprint": product_footprint_verified.model_dump()
        }

        log_task_completion("hub_procedure")

        return result

    def define_product_footprint_template(self, company_name: str, shipment_information: dict) -> dict:
        """
        Define a product footprint.

        Returns:
            Dictionary containing the product footprint
        """
        log_task_start("define_product_footprint_template")

        product_footprint = ProductFootprint(
            id=str(uuid.uuid4()),
            created=datetime.datetime.now().isoformat(),
            specVersion="2.0.0",
            version=0,
            status="Active",
            companyName=company_name,
            companyIds=[f"urn:epcidsgln:{uuid.uuid4()}"],
            productDescription=f"Logistics emissions related to shipment with ID {shipment_information.get('shipment_id', 'unknown')}",
            productIds=[
                f"urn:pathfinder:product:customcode:vendor-assigned:{uuid.uuid4()}"],
            productCategoryCpc=random.randint(1000, 9999),
            productNameCompany=f"Shipment with ID {shipment_information.get('shipment_id', 'unknown')}",
            extensions=[
                Extension(
                    dataSchema="https://api.ileap.sine.dev/shipment-footprint.json",
                    data=ExtensionData(
                        mass=shipment_information.get(
                            "shipment_weight", random.uniform(1000, 20000)),
                        shipmentId=shipment_information.get(
                            "shipment_id", f"SHIP_{uuid.uuid4()}")
                    )
                )
            ]
        )
        result = {
            "product_footprint": product_footprint.model_dump()
        }

        log_task_completion("define_product_footprint_template")
        return result

    def determine_job_sequence(self):
        """
        Determine which subprocesses should be executed.

        Returns:
            Dictionary containing the list of subprocess identifiers
        """
        log_task_start("determine_job_sequence")

        subprocesses = [
            "case_1_with_tsp",
            "case_2_with_tsp",
            "case_3_with_tsp",
        ]
        result = {"subprocess_identifiers": subprocesses}

        log_task_completion("determine_job_sequence", **result)
        return result

    def call_service_sensordata(self):
        pass

    def call_service_sensordata_certificate(self):
        pass

    def send_to_proofing_service(self, proofing_document: dict, product_footprint: dict) -> dict:
        # call proofing service by api
        product_footprint_reference = product_footprint

        return {"product_footprint": product_footprint_reference}

    async def notify_next_node(self, message_name: str, shipment_information: dict) -> None:
        """
        Publish a message to notify the next node in the process.

        Args:
            message_name: Name of the message to publish
            shipment_information: Information about shipment and weight
        """
        log_task_start("notify_next_node",
                       message_name=message_name, shipment_information=shipment_information)

        # Publish the message
        await self.client.publish_message(
            name=message_name,
            correlation_key=f"{message_name}-{shipment_information.get('shipment_id', 'unknown')}",
            variables={"shipment_information": shipment_information}
        )

        log_task_completion("notify_next_node")

    async def send_data_to_origin(
            self,
            shipment_information: dict,
            message_name: str,
            product_footprints: dict,
    ):
        """
        Send data back to the origin process.

        Args:
            shipment_information: Information about shipment and weight
            message_name: Name of the message to publish
            tce_data: Tce data to include in the message
        """
        log_task_start("send_data_to_origin",
                       shipment_information=shipment_information, message_name=message_name)

        await self.client.publish_message(
            name=message_name,
            correlation_key=shipment_information.get("shipment_id", "unknown"),
            variables={
                "shipment_id": shipment_information.get("shipment_id", "unknown"),
                "product_footprints": product_footprints
            }
        )

        log_task_completion("send_data_to_origin")

    def set_shipment_information(self):
        """
        Generate a new shipment ID.
        And weight for the shipment.

        Returns:
            Dictionary containing the new shipment ID and weight
        """
        log_task_start("set_shipment_information")

        shipment_id = f"SHIP_{uuid.uuid4()}"
        weight = random.uniform(1000, 20000)
        result = {"shipment_information": {
            "shipment_id": shipment_id, "shipment_weight": weight}}

        log_task_completion("set_shipment_information", **result)
        return result
