-- Add indices to `reservation`.`profile_id` and `reservation`.`opening_time_id`
CREATE INDEX idx__reservation__profile_id ON reservation (profile_id);
CREATE INDEX idx__reservation__opening_time_id ON reservation (opening_time_id);
CREATE INDEX idx__reservation__confirmed_by ON reservation (confirmed_by);