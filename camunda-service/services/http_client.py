import json
import logging
from typing import Dict, Any, Optional

import aiohttp

logger = logging.getLogger("camunda_service")

class HttpClient:
    """Generic HTTP client for external API calls."""
    
    def __init__(self, base_url: str, headers: Optional[Dict[str, str]] = None):
        """
        Initialize HTTP client with base URL and default headers.
        
        Args:
            base_url: The base URL for all requests
            headers: Optional default headers for all requests
        """
        self.base_url = base_url
        self.headers = headers or {}
    
    async def get(self, endpoint: str, params: Optional[Dict[str, Any]] = None) -> Dict:
        """
        Make a GET request to the specified endpoint.
        
        Args:
            endpoint: The API endpoint
            params: Optional query parameters
            
        Returns:
            The response data as a dictionary
        """
        url = f"{self.base_url}/{endpoint}"
        logger.debug(f"Making GET request to {url}")
        
        async with aiohttp.ClientSession() as session:
            async with session.get(url, params=params, headers=self.headers) as response:
                response.raise_for_status()
                return await response.json()
    
    async def post(self, endpoint: str, data: Dict[str, Any]) -> Dict:
        """
        Make a POST request to the specified endpoint.
        
        Args:
            endpoint: The API endpoint
            data: The data to send in the request body
            
        Returns:
            The response data as a dictionary
        """
        url = f"{self.base_url}/{endpoint}"
        logger.debug(f"Making POST request to {url}")
        
        async with aiohttp.ClientSession() as session:
            async with session.post(
                url, 
                data=json.dumps(data), 
                headers={**self.headers, "Content-Type": "application/json"}
            ) as response:
                response.raise_for_status()
                return await response.json()
    
    async def put(self, endpoint: str, data: Dict[str, Any]) -> Dict:
        """
        Make a PUT request to the specified endpoint.
        
        Args:
            endpoint: The API endpoint
            data: The data to send in the request body
            
        Returns:
            The response data as a dictionary
        """
        url = f"{self.base_url}/{endpoint}"
        logger.debug(f"Making PUT request to {url}")
        
        async with aiohttp.ClientSession() as session:
            async with session.put(
                url, 
                data=json.dumps(data), 
                headers={**self.headers, "Content-Type": "application/json"}
            ) as response:
                response.raise_for_status()
                return await response.json()