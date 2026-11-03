from .base import BaseUserTest, PlainMixin


class RelativeTest(BaseUserTest):
    def a(self):
        pass

    def b(self):
        pass

    def c(self):
        pass


class MixinTest(PlainMixin):
    def a(self):
        pass

    def b(self):
        pass

    def c(self):
        pass
