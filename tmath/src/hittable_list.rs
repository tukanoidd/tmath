macro_rules! decl_hittable_lists {
    ($hittable_list:ident => [$(($var_ty:ty, $variant:ident)),*]) => {
	    #[derive(Default, Clone)]
		pub struct $hittable_list {
		    pub objects: Vec<std::sync::Arc<dyn crate::hittable::Hittable + Sync + Send>>,
		}

		impl $hittable_list {
		    #[inline]
		    pub fn new(
			    object: std::sync::Arc<dyn crate::hittable::Hittable + Sync + Send>
		    ) -> Self {
		        Self {
		            objects: vec![object],
		        }
		    }

		    #[inline]
		    pub fn clear(&mut self) {
		        self.objects.clear();
		    }

		    #[inline]
		    pub fn add(
			    &mut self,
			    object: std::sync::Arc<dyn crate::hittable::Hittable + Sync + Send>
		    ) {
		        self.objects.push(object);
		    }
		}

		impl crate::hittable::Hittable for $hittable_list {
		    fn hit(
		        &self,
		        r: &crate::ray::Ray,
		        t_min: f32,
		        t_max: f32,
		        rec: &mut crate::hittable::HitRecord,
		    ) -> bool {
		        let mut temp_rec = crate::hittable::HitRecord::default();
		        let mut hit_anything = false;
		        let mut closest_so_far = t_max;

		        for object in &self.objects {
		            if object.hit(r, t_min, closest_so_far, &mut temp_rec) {
		                hit_anything = true;
		                closest_so_far = temp_rec.t;
		                *rec = temp_rec;
		            }
		        }

		        hit_anything
		    }
		}

	    paste::paste! {
		    $(
		        #[derive(Default, Clone)]
				pub struct [< $hittable_list $variant >] {
				    pub objects: Vec<
				        std::sync::Arc<
				            dyn crate::hittable::[< Hittable $variant >] + Sync + Send
				        >
				    >,
				}

				impl [< $hittable_list $variant >] {
				    #[inline]
				    pub fn new(
					    object: std::sync::Arc<
					        dyn crate::hittable::[< Hittable $variant >] + Sync + Send
					    >
				    ) -> Self {
				        Self {
				            objects: vec![object],
				        }
				    }

				    #[inline]
				    pub fn clear(&mut self) {
				        self.objects.clear();
				    }

				    #[inline]
				    pub fn add(
					    &mut self,
					    object: std::sync::Arc<
					        dyn crate::hittable::[< Hittable $variant >] + Sync + Send
					    >
				    ) {
				        self.objects.push(object);
				    }
				}

				impl crate::hittable::[< Hittable $variant >] for [< $hittable_list $variant >] {
				    fn hit(
				        &self,
				        r: &crate::ray::[< Ray $variant >],
				        t_min: $var_ty,
				        t_max: $var_ty,
				        rec: &mut crate::hittable::[< HitRecord $variant >],
				    ) -> bool {
				        let mut temp_rec = crate::hittable::[< HitRecord $variant >]::default();
				        let mut hit_anything = false;
				        let mut closest_so_far = t_max;

				        for object in &self.objects {
				            if object.hit(r, t_min, closest_so_far, &mut temp_rec) {
				                hit_anything = true;
				                closest_so_far = temp_rec.t;
				                *rec = temp_rec;
				            }
				        }

				        hit_anything
				    }
				}
		    )*
	    }
    };
}

decl_hittable_lists!(HittableList => [(f64, D)]);
