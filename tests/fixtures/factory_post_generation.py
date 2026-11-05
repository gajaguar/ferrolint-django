"""factory_boy `post_generation` methods take the instance, not `self`."""
import factory
import factory as fb
from factory import post_generation
from factory.helpers import post_generation as post_hook


class UserFactory(factory.Factory):
    @post_generation
    def groups(obj, create, extracted, **kwargs):
        return None

    @factory.post_generation
    def tags(obj, create, extracted, **kwargs):
        return None

    @factory.helpers.post_generation
    def roles(obj, create, extracted, **kwargs):
        return None

    @fb.post_generation
    def aliased(obj, create, extracted, **kwargs):
        return None

    @post_hook
    def renamed(obj, create, extracted, **kwargs):
        return None

    @staticmethod
    @factory.post_generation
    def stacked(obj, create, extracted, **kwargs):
        return None

    @property
    def control(obj):
        return None
