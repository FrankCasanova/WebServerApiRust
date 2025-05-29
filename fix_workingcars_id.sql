CREATE SEQUENCE IF NOT EXISTS "WorkingCars_id_seq";
ALTER TABLE "WorkingCars" ALTER COLUMN id SET DEFAULT nextval('"WorkingCars_id_seq"');
