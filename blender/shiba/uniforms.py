import bpy
import ctypes
from dataclasses import dataclass, field
from shiba import api, callback_lists
from typing import List, Optional


class Uniforms(bpy.types.PropertyGroup):
    pass


@dataclass
class UniformAnnotationDescriptor:
    pass


@dataclass
class UniformAnnotationControlDescriptor(UniformAnnotationDescriptor):
    name: str = None

    @property
    def property_name(self):
        return 'uniform_%s' % self.name

    def add_property(self):
        pass

    def remove_property(self):
        pass

    def draw_property(self, uniforms, layout):
        pass

    def to_api_uniform_value(self, uniforms):
        pass


@dataclass
class UniformAnnotationControlCheckboxDescriptor(UniformAnnotationControlDescriptor):
    default: bool = False

    def add_property(self):
        setattr(Uniforms, self.property_name, bpy.props.BoolProperty(
            name=self.name,
            default=self.default,
            update=_update,
        ))

    def remove_property(self):
        delattr(Uniforms, self.property_name)

    def draw_property(self, uniforms, layout):
        layout.prop(uniforms, self.property_name)

    def to_api_uniform_value(self, uniforms):
        return api.UniformValue(
            as_int=getattr(uniforms, self.property_name),
        )


@dataclass
class UniformAnnotationControlObjectDescriptor(UniformAnnotationControlDescriptor):
    def to_api_uniform_value(self, uniforms):
        return api.UniformValue()


@dataclass
class UniformAnnotationControlSliderDescriptor(UniformAnnotationControlDescriptor):
    default: Optional[float] = None
    min: Optional[float] = None
    max: Optional[float] = None

    def to_api_uniform_value(self, uniforms):
        return api.UniformValue()


@dataclass
class UniformDescriptor:
    annotations: List[UniformAnnotationDescriptor] = field(
        default_factory=list)
    name: str = None
    type_name: str = None

    def get_annotation(self, Class):
        for annotation in self.annotations:
            if isinstance(annotation, Class):
                return annotation
        return None


_active_uniform_descriptors = []


def get_active_uniform_descriptors():
    return _active_uniform_descriptors


def _update(_self, _context):
    callback_lists.viewport_update.trigger()


def set_api_active_uniform_descriptors(api_active_uniform_count, api_active_uniform_descriptors):
    _active_uniform_descriptors.clear()

    def make_annotation(api_descriptor_pointer, name):
        api_descriptor = ctypes.cast(
            api_descriptor_pointer,
            ctypes.POINTER(api.UniformAnnotationDescriptor)
        ).contents
        if api_descriptor.kind == api.UNIFORM_ANNOTATION_KIND_CONTROL:
            api_descriptor = ctypes.cast(
                api_descriptor_pointer,
                ctypes.POINTER(api.UniformAnnotationControlDescriptor)
            ).contents
            if api_descriptor.control_kind == api.UNIFORM_ANNOTATION_CONTROL_KIND_CHECKBOX:
                return UniformAnnotationControlCheckboxDescriptor(
                    name=name,
                )
            if api_descriptor.control_kind == api.UNIFORM_ANNOTATION_CONTROL_KIND_OBJECT:
                return UniformAnnotationControlObjectDescriptor()
            if api_descriptor.control_kind == api.UNIFORM_ANNOTATION_CONTROL_KIND_SLIDER:
                return UniformAnnotationControlSliderDescriptor()
        raise NotImplementedError()

    for descriptor_index in range(api_active_uniform_count):
        api_uniform_descriptor = api_active_uniform_descriptors[descriptor_index]
        name = api_uniform_descriptor.name.decode()
        type_name = api_uniform_descriptor.type_name.decode()
        uniform_descriptor = UniformDescriptor(
            annotations=[
                make_annotation(
                    api_uniform_descriptor.annotations[annotation_index], name)
                for annotation_index in range(api_uniform_descriptor.annotation_count)
            ],
            name=name,
            type_name=type_name,
        )
        _active_uniform_descriptors.append(uniform_descriptor)

        annotation_control = uniform_descriptor.get_annotation(
            UniformAnnotationControlDescriptor)
        if annotation_control is not None:
            annotation_control.add_property()


def get_api_uniform_values(uniforms):
    def to_api_uniform_value(uniform_descriptor):
        annotation_control = uniform_descriptor.get_annotation(
            UniformAnnotationControlDescriptor)
        if annotation_control is not None:
            return annotation_control.to_api_uniform_value(uniforms)
        return api.UniformValue()

    UniformValueArray = api.UniformValue * len(_active_uniform_descriptors)
    array = [
        to_api_uniform_value(uniform_descriptor)
        for uniform_descriptor in _active_uniform_descriptors
    ]
    uniform_values = UniformValueArray(*array)

    return uniform_values
