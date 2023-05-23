# Objective: Script that runs on a timer schedule to create messages to be sent into the MQTT broker
import tomli
import logging
import typing
from pyproto import messages_pb2 as Messages
import datetime
import paho.mqtt.publish as Publish 

# Loading the config file
with open("python/conf/amzn.toml", "rb") as fp:
    config = tomli.load(fp)
    try:
        logger_config: typing.Dict[str, typing.Union[int, str]] = config["logger"]
        products_config: typing.Dict[str, typing.Union[str, typing.List[str]]] = config[
            "products"
        ]
        mqtt_config: typing.Dict[str, str] = config["mqtt"]
    except Exception as e:
        logging.error(f"Unable to load the config file. Terminating now.")
        exit()

# Loading the logger configuration
try:
    logging.basicConfig(
        level=int(logger_config["level"]),
        format=str(logger_config["format"]),
        datefmt=str(logger_config["datefmt"]),
    )
    logging.info(f"Logger setup completed successfully.")
except Exception as e:
    logging.error(
        f"Unable to setup the logger properly due to the following error: {e}."
    )

# Constructing the MQTT message
asin_codes: typing.Optional[typing.Union[str, typing.List[str]]] = products_config.get(
    "asin"
)
if not isinstance(asin_codes, typing.List):
    raise TypeError(f"Expected a type of List[str], got {type(asin_codes)} instead.")

msg: Messages.AmznScrapingRequest = Messages.AmznScrapingRequest(
    request_timestamp=int(datetime.datetime.utcnow().timestamp()),
    product_codes=asin_codes
)

# Publishing the message
try:
    msg.SerializeToString()
    publish_result = Publish.single(
        topic=mqtt_config["topic"],
        client_id=mqtt_config["client_id"]
    )
except Exception as e:
    logging.error(f"Unable to publish message due to the following error: {e}.")