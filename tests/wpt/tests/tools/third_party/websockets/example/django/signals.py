import json

from django.contrib.admin.models import LogEntry
from django.db.models.signals import post_save
from django.dispatch import receiver

from django_redis import get_redis_connection


@receiver(post_save, sender=LogEntry)
def publish_event(instance, **kwargs):
    event = {
        "model": instance.content_type.name,
        "object": instance.object_repr,
        "message": instance.get_change_message(),
        "timestamp": instance.action_time.isoformat(),
        "user": str(instance.user),
        "content_type_id": instance.content_type_id,
        "object_id": instance.object_id,
    }
    connection = get_redis_connection("default")
    payload = json.dumps(event)
    connection.publish("events", payload)
