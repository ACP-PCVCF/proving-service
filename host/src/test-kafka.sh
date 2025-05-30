#!/bin/bash

INPUT_TOPIC="shipments"
KAFKA_CONTAINER="host-kafka-1"
BROKER="localhost:9092"

# Multi-Line JSON in Variable
read -r -d '' TEST_MESSAGE << 'EOF'
[
  {
    "shipment_id": "SHIP_3a9eb761-c4b3-40c1-8658-5652211d7367",
    "info": {
      "activity_data_json": "{\"arrivalAt\":\"2025-05-08T03:05:21Z\",\"co2_factor_ttw_per_tkm\":\"0.095\",\"co2eTTW\":\"559.625\",\"co2eWTW\":\"654.271\",\"departureAt\":\"2025-05-07T21:34:27Z\",\"destination\":{\"city\":\"München\",\"countryCode\":\"DE\",\"locationName\":\"München\",\"postalCode\":\"67805\",\"type\":\"PhysicalLocation\"},\"distance\":{\"dataSource\":\"Simulated\",\"unit\":\"km\",\"value\":\"417.94\"},\"hocId\":\"HOC_02454dc6-0cd3-435e-8456-e3edeba82baf\",\"incoterms\":\"FCA\",\"mass\":\"14166.32\",\"origin\":{\"city\":\"Leipzig\",\"countryCode\":\"DE\",\"locationName\":\"Leipzig\",\"postalCode\":\"52963\",\"type\":\"PhysicalLocation\"},\"packagingOrTrEqAmount\":\"9.17\",\"packagingOrTrEqType\":\"Bulk\",\"shipmentId\":\"SHIP_1dc6b75e-74d1-41f0-9b21-737b558bbb5f\",\"tceId\":\"TCE_5db5deda-d1d8-4614-a671-6185cc0e326e\",\"transportActivity\":\"5920.722\",\"wtw_multiplier\":\"1.169\"}",
      "activity_signature": "Uzp6gzcTUG0LuU9CWysz8TIsmVDQiS1+HBJrm9eiSCpK4luMgmAfhWkxkK33mM+bojsdgOcKuladAee6+GFQgrTCeoSsqtxpfsgVp31u8f0MxqYv7jqd59ofR737f4anlVp5pzLpj2UFrK+40TnLdIckcOtOxegf5NzztRSEV/bY26/iku0/DRqenqbPifEj9AsVbdms+C50bXMchn/kMQKmybOZArwy0u29X+97PhycFOIC94pLCv2z+0oL7DKEY/TdQDCToZdBDxLZrKduvyriKXeEOsoOS359ElWwIdXEta4w0mooEe4SMXJ2IIarNNS45rHjLYIuF7PrLhRzbA==",
      "activity_public_key_pem": "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEArctYpdG5jXRW0FDAFX0WSPOpmJ7vXkdjcTKGt5u+3ndmBULKK00l\nAMRoz+zcYaAReE6TC2U0h8/+Vg71JNxu2JhBVDZ3iMlvZSkKb9SDWSN8abkP5Sx+\nZ25xYCnU+23bBtpRRs3uxw4kCbY52nGFteRhxDO0iLeL15cs+SEMov36CauDwPmS\nMJNvoBEFZCR4OmvNCmrAf7ToYr6MaLRUv5Th3ygtACtgcZI5eyK9ti/bb2yI6ani\nGUIJSfb2PN0kB75PN6YssPrzhNAmZVCHyEQ73Z+etQXbx22oNK+lNpjr3Pm+xf5h\nnPdyJTrrAw2AJZRncM7SnVK+9iVRv+FF0wIDAQAB\n-----END RSA PUBLIC KEY-----"
    }
  },
  {
    "shipment_id": "SHIP_44952dce-316f-4eee-9c68-3dcf6d8a1bbe",
    "info": {
      "activity_data_json": "{\"arrivalAt\":\"2025-05-08T03:53:32Z\",\"co2_factor_ttw_per_tkm\":\"0.100\",\"co2eTTW\":\"591.140\",\"co2eWTW\":\"713.147\",\"departureAt\":\"2025-05-07T21:37:53Z\",\"destination\":{\"city\":\"Mannheim\",\"countryCode\":\"DE\",\"locationName\":\"Mannheim\",\"postalCode\":\"99952\",\"type\":\"PhysicalLocation\"},\"distance\":{\"dataSource\":\"Simulated\",\"unit\":\"km\",\"value\":\"417.94\"},\"incoterms\":\"FAS\",\"mass\":\"14166.32\",\"noxTTW\":\"11.8609\",\"origin\":{\"city\":\"Berlin\",\"countryCode\":\"DE\",\"locationName\":\"Berlin\",\"postalCode\":\"96154\",\"type\":\"PhysicalLocation\"},\"shipmentId\":\"SHIP_1dc6b75e-74d1-41f0-9b21-737b558bbb5f\",\"tceId\":\"TCE_cfdcee37-8171-4032-956c-2b5f3825a7a5\",\"tocId\":\"TOC_1d16c82f-008a-47c0-8641-d3a725520877\",\"transportActivity\":\"5920.722\",\"wtw_multiplier\":\"1.206\"}",
      "activity_signature": "ofEbfExklDhEwTrTLzJAOIBwG4NkVo+kWeHE0lnmCOmI2XLjec62UOCPhNeGjrWESlFDG1HdGhpNgglDVr63luPPDR7+Slc1vDxSWHLar79cD1v16h+zD42DKBmdyDhGG9Yxmkhu5bY3qQqs0UCaHkdpxb5VjtGBR+/j/w/WGqguDgW7rwmE4u88wOsVNFZ4oLuDlzFAt7Hch0O2/FDPN1Y7ct5AkNLxOso7af8twHdDgYpWxobi7X/1vW7Ry/mwsL0uOEirJJgd1N+Tc//anrh7fs/4F/6aNY/87NNY+mc/aGHUfYsZHKwzBfG8viVKvFVOa2egYJYuRkB+0shYwQ==",
      "activity_public_key_pem": "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEArctYpdG5jXRW0FDAFX0WSPOpmJ7vXkdjcTKGt5u+3ndmBULKK00l\nAMRoz+zcYaAReE6TC2U0h8/+Vg71JNxu2JhBVDZ3iMlvZSkKb9SDWSN8abkP5Sx+\nZ25xYCnU+23bBtpRRs3uxw4kCbY52nGFteRhxDO0iLeL15cs+SEMov36CauDwPmS\nMJNvoBEFZCR4OmvNCmrAf7ToYr6MaLRUv5Th3ygtACtgcZI5eyK9ti/bb2yI6ani\nGUIJSfb2PN0kB75PN6YssPrzhNAmZVCHyEQ73Z+etQXbx22oNK+lNpjr3Pm+xf5h\nnPdyJTrrAw2AJZRncM7SnVK+9iVRv+FF0wIDAQAB\n-----END RSA PUBLIC KEY-----"
    }
  }
]
EOF

# Komprimiere in eine einzige Zeile
SINGLE_LINE_JSON=$(printf '%s' "$TEST_MESSAGE" | tr -d '\n' | tr -s ' ')

echo "Sende Test-Nachricht an Topic $INPUT_TOPIC ..."
printf '%s\n' "$SINGLE_LINE_JSON" | \
  docker exec -i $KAFKA_CONTAINER \
    kafka-console-producer --broker-list $BROKER --topic $INPUT_TOPIC > /dev/null

echo "Nachricht gesendet."
